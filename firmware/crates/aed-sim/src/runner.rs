// SPDX-License-Identifier: GPL-3.0-or-later
//! Runs one scenario: the real rescue sequence and the real supervisor,
//! ticking against the simulated plant, with the safety invariants checked
//! at every step and a timeline recorded.
//!
//! Timing: the plant advances in 1 ms steps; the main controller ticks every
//! 2 ms and receives a heart-signal sample every 4 ms (250 per second); the
//! supervisor ticks every 1 ms. These rates are modelling choices.

use std::cell::RefCell;
use std::rc::Rc;

use aed_core::{
    AbortReason, Config, Event, FaultKind, PermitRefusal, Prompt, ReleaseReason, Rescue,
    RescueState, ShockOutcome, Supervisor, SupervisorFault,
};
use aed_cpr::CompressionDetector;
use aed_rhythm::synth::Synth;
use aed_rhythm::Analyzer;
use aed_types::{PatientMode, RhythmDecision, MAX_PATIENT_OHMS, MIN_PATIENT_OHMS};
use aed_waveform::BtePlanner;

use crate::hw::{MainView, SharedPlant, SupervisorView};
use crate::plant::{rhythm_is_shockable, rhythm_name, Fault, Plant, PlantEvent, ShockRecord};
use crate::scenario::{Action, Scenario, Trigger};

/// How far above the overvoltage limit the true capacitor voltage may go
/// before the invariant fails, volts. The limit is checked by software on a
/// tick, so a small overshoot between ticks is expected. Model choice: 50 V.
pub const OVERVOLTAGE_MARGIN_V: f64 = 50.0;

/// Main controller tick period, ms.
pub const MAIN_TICK_MS: u32 = 2;
/// Heart-signal sample period, ms (250 samples per second).
pub const ECG_PERIOD_MS: u32 = 4;
/// Pad-impedance sample period, ms (125 samples per second).
pub const IMPEDANCE_PERIOD_MS: u32 = 8;
/// With compression sensing, the bridge may fire while hands are on the
/// chest only if they went on less than this long before: the time the
/// detector needs to see the first compression, plus a sample and a tick.
/// 1,000 ms: the detector's design bound for a compression of at least
/// 0.3 Ω at 50 per minute or faster (aed-cpr crate documentation). The
/// exposures actually seen are reported in every run.
pub const HANDS_ON_ALLOWANCE_MS: u32 = 1_000;

/// Who or what a timeline line is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The device spoke or showed a prompt.
    Said,
    /// The main controller changed state or recorded something.
    Device,
    /// The rhythm analysis gave an answer.
    Analysis,
    /// The supervisor did something.
    Supervisor,
    /// Something physical in the device (capacitor, bridge).
    Hardware,
    /// The patient (rhythm, pads).
    Patient,
    /// The person doing the rescue.
    Responder,
    /// A fault was injected by the scenario.
    Injected,
    /// A safety invariant failed.
    Violation,
}

impl Source {
    /// A fixed-width label for printing.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Source::Said => "SAYS      ",
            Source::Device => "device    ",
            Source::Analysis => "analysis  ",
            Source::Supervisor => "supervisor",
            Source::Hardware => "hardware  ",
            Source::Patient => "patient   ",
            Source::Responder => "responder ",
            Source::Injected => "INJECTED  ",
            Source::Violation => "VIOLATION ",
        }
    }
}

/// One line of the timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Simulation time, ms.
    pub t_ms: u32,
    /// Who or what.
    pub source: Source,
    /// What happened, in plain English.
    pub text: String,
}

/// The safety rules checked during every run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rule {
    /// The bridge fired while the patient's true rhythm was not VF/VT.
    FiredOnNonShockableRhythm,
    /// The bridge fired with the pads off or the true resistance out of range.
    FiredWithPadsOffOrOutOfRange,
    /// The bridge fired while the cancel control was pressed.
    FiredWhileCancelPressed,
    /// The bridge fired while the main controller was not in its Delivering state.
    FiredOutsideDelivering,
    /// The true capacitor voltage went above the overvoltage limit plus the margin.
    Overvoltage,
    /// The device ended in its Fault state with the capacitor still charged.
    FaultLeftCharged,
    /// The discharge path was stuck, the capacitor stayed charged, and the
    /// device did not report the serious "could not empty" fault.
    StuckDischargeNotReportedSerious,
    /// The capacitor was left charged at the end while no shock was being prepared.
    CapacitorLeftCharged,
    /// An informational prompt (for example "Shock delivered.") was not
    /// followed by an instruction within [`silence_grace_ms`], or the device
    /// never said anything.
    LeftWithoutInstruction,
    /// The device said nothing at all for longer than twice the grace time
    /// after switch-on.
    SilentAtPowerOn,
    /// With compression sensing, the bridge fired while the responder's
    /// hands had been on the chest for longer than [`HANDS_ON_ALLOWANCE_MS`].
    FiredWhileHandsOn,
}

impl Rule {
    /// Plain-English statement of the rule.
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Rule::FiredOnNonShockableRhythm => "bridge fired on a rhythm that was not VF/VT",
            Rule::FiredWithPadsOffOrOutOfRange => {
                "bridge fired with pads off or resistance out of range"
            }
            Rule::FiredWhileCancelPressed => "bridge fired while cancel was pressed",
            Rule::FiredOutsideDelivering => "bridge fired while main was not delivering",
            Rule::Overvoltage => "capacitor above the overvoltage limit plus margin",
            Rule::FaultLeftCharged => "Fault state with the capacitor still charged",
            Rule::StuckDischargeNotReportedSerious => {
                "discharge stuck but serious fault not reported"
            }
            Rule::CapacitorLeftCharged => "capacitor left charged at the end of the run",
            Rule::LeftWithoutInstruction => "responder left without an instruction",
            Rule::SilentAtPowerOn => "device silent after switch-on",
            Rule::FiredWhileHandsOn => "bridge fired while hands were on the chest",
        }
    }
}

/// One broken safety rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    /// When, ms.
    pub t_ms: u32,
    /// Which rule.
    pub rule: Rule,
    /// Details.
    pub detail: String,
    /// For [`Rule::FiredOnNonShockableRhythm`]: how long before the shock the
    /// rhythm had stopped being shockable, ms.
    pub rhythm_changed_ms_before: Option<u32>,
}

/// The timings of one shock, from the start of the analysis that led to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShockTiming {
    /// 1 for the first shock, and so on.
    pub number: usize,
    /// When the bridge fired, ms.
    pub at_ms: u32,
    /// Analysis start to "shock advised", ms.
    pub analysis_ms: u32,
    /// Charging start to "charged", ms. With charging during analysis
    /// (D-29) this overlaps the analysis.
    pub charge_ms: u32,
    /// "Shock advised" to the start of the countdown: the wait for the
    /// charge to finish, ms. Zero when the capacitor charged during the
    /// analysis and was ready.
    pub charge_wait_ms: u32,
    /// Countdown start to shock request, ms.
    pub countdown_ms: u32,
    /// Shock request to the bridge firing (waiting for the supervisor's permit), ms.
    pub permit_wait_ms: u32,
    /// From the moment compressions stopped to the shock, ms. `None` if
    /// compressions never stopped (should not happen).
    pub hands_off_ms: Option<u32>,
}

/// One armed cycle being tracked.
#[derive(Debug, Clone, Copy, Default)]
struct Cycle {
    analysis_start: u32,
    charge_start: Option<u32>,
    charged: Option<u32>,
    advised: Option<u32>,
    countdown_start: Option<u32>,
    request: Option<u32>,
}

/// Everything a run produced.
#[derive(Debug, Clone)]
pub struct RunResult {
    /// The scenario that was run.
    pub scenario: Scenario,
    /// The timeline, in time order.
    pub timeline: Vec<Entry>,
    /// Every prompt, with its time.
    pub prompts: Vec<(u32, Prompt)>,
    /// Every state the main controller entered, with the time.
    pub states: Vec<(u32, RescueState)>,
    /// Every rhythm decision, with its time.
    pub decisions: Vec<(u32, RhythmDecision)>,
    /// Every shock the bridge fired.
    pub shocks: Vec<ShockRecord>,
    /// Timing of each shock.
    pub timings: Vec<ShockTiming>,
    /// Broken safety rules.
    pub violations: Vec<Violation>,
    /// The main controller's state at the end.
    pub final_state: RescueState,
    /// The main controller's fault, if any.
    pub final_fault: Option<FaultKind>,
    /// The supervisor's latched fault, if any.
    pub supervisor_fault: Option<SupervisorFault>,
    /// True capacitor voltage at the end.
    pub final_cap_v: f64,
    /// Highest true capacitor voltage at any time.
    pub max_cap_v: f64,
    /// Faults injected, with their times.
    pub injected: Vec<(u32, Fault)>,
    /// When the pads first went on, ms.
    pub pads_on_at_ms: Option<u32>,
    /// How many times charging for a shock started (with charging during
    /// analysis, D-29: once per analysis, plus a recharge if the adult/child
    /// switch is moved during one).
    pub charges_started: u32,
    /// Charges emptied into the internal load without a shock (the main
    /// controller's own count).
    pub charges_burned: u16,
    /// Energy the charger put into the capacitor over the run, joules (model).
    pub charger_energy_j: f64,
    /// Energy burned off in the internal discharge load over the run,
    /// joules (model): unused charges, what is left after shocks, and the
    /// self-test charge.
    pub dump_energy_j: f64,
    /// The supervisor refused a permit at least once for these reasons.
    pub supervisor_refusals: Vec<PermitRefusal>,
    /// Metronome beats.
    pub metronome_beats: u32,
    /// Messages lost on the wire.
    pub messages_dropped: u32,
    /// When the supervisor's own fault alarm first came on, ms (`None`: never).
    pub supervisor_alarm_at: Option<u32>,
    /// When the main controller froze, ms (`None`: it did not).
    pub main_frozen_at: Option<u32>,
    /// The configuration both controllers used.
    pub config: Config,
    /// For every shock fired while hands were on the chest (ground truth):
    /// how long they had been on, ms.
    pub hands_on_at_fire: Vec<u32>,
    /// Compressions the firmware counted.
    pub compressions_counted: u32,
    /// Times the firmware held a shock back because the chest was not still.
    pub hands_on_aborts: u16,
}

impl RunResult {
    /// Time from the pads going on to the first shock, ms.
    #[must_use]
    pub fn pads_on_to_first_shock_ms(&self) -> Option<u32> {
        let on = self.pads_on_at_ms?;
        self.shocks.first().map(|s| s.at_ms.saturating_sub(on))
    }

    /// True if the device ever entered this state.
    #[must_use]
    pub fn entered(&self, s: RescueState) -> bool {
        self.states.iter().any(|&(_, x)| x == s)
    }

    /// The last prompt, if any.
    #[must_use]
    pub fn last_prompt(&self) -> Option<Prompt> {
        self.prompts.last().map(|&(_, p)| p)
    }

    /// True if this prompt was ever given.
    #[must_use]
    pub fn said(&self, p: Prompt) -> bool {
        self.prompts.iter().any(|&(_, x)| x == p)
    }

    /// Violations of one rule.
    #[must_use]
    pub fn violations_of(&self, rule: Rule) -> Vec<&Violation> {
        self.violations.iter().filter(|v| v.rule == rule).collect()
    }
}

/// True for prompts that tell the responder what to do (or not to do).
#[must_use]
pub const fn is_instruction(p: Prompt) -> bool {
    matches!(
        p,
        Prompt::AttachPads
            | Prompt::ContinueCpr
            | Prompt::CheckPads
            | Prompt::AnalysingDoNotTouch
            | Prompt::StopCprAnalysing
            | Prompt::DoNotTouchPatient
            | Prompt::MovementShockDelayed
            | Prompt::ShockAdvisedCharging
            | Prompt::StandClear
            | Prompt::Countdown(_)
            | Prompt::StartCpr
            | Prompt::DeviceFaultContinueCpr
            | Prompt::SeriousFaultContinueCpr
            | Prompt::CancelStuckContinueCpr
            | Prompt::PressPadsFirmly
            | Prompt::ChildModeFrontBack
    )
}

/// Prompts that neither instruct nor replace the current instruction: the
/// responder keeps doing what they were last told.
#[must_use]
pub const fn is_neutral(p: Prompt) -> bool {
    matches!(p, Prompt::LowBattery | Prompt::AdultMode | Prompt::ReplacePads)
}

/// How long an informational prompt may stand before an instruction must
/// follow: the disarm time limit (emptying after an abort) plus 1 s.
#[must_use]
pub const fn silence_grace_ms(config: &Config) -> u32 {
    config.disarm_time_limit_ms.saturating_add(1_000)
}

/// The whole simulated system: plant, both controllers, the responder and the script.
pub struct Simulation {
    scenario: Scenario,
    plant: SharedPlant,
    rescue: Rescue<Analyzer, BtePlanner, CompressionDetector>,
    supervisor: Supervisor,
    hands_on_at_fire: Vec<u32>,
    main_view: MainView,
    sup_view: SupervisorView,
    config: Config,
    main_frozen: bool,
    main_frozen_at: Option<u32>,
    alarm_on: bool,
    alarm_first_at: Option<u32>,
    script_done: Vec<bool>,
    timeline: Vec<Entry>,
    states: Vec<(u32, RescueState)>,
    decisions: Vec<(u32, RhythmDecision)>,
    violations: Vec<Violation>,
    injected: Vec<(u32, Fault)>,
    prompts_seen: usize,
    last_directive: Option<(u32, Prompt)>,
    silence_reported: bool,
    main_events_seen: u64,
    sup_events_seen: u64,
    last_printed_decision: Option<RhythmDecision>,
    responder_pending: Option<(u32, bool)>,
    reattach_at: Option<u32>,
    press_pads_at: Option<u32>,
    pads_on_at: Option<u32>,
    hands_off_since: Option<u32>,
    cycle: Option<Cycle>,
    timings: Vec<ShockTiming>,
    charges_started: u32,
    refusals: Vec<PermitRefusal>,
    milestone_target: Option<u16>,
    milestone_next: u8,
    cap_was_charged: bool,
    overvoltage_reported: bool,
}

/// Why a simulation could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    /// The firmware configuration was refused.
    Config(aed_core::ConfigError),
    /// The waveform configuration was refused.
    Waveform(aed_waveform::ConfigError),
    /// The compression detector's settings were refused.
    Chest(aed_cpr::SettingsError),
}

impl Simulation {
    /// Build the system for a scenario, using the firmware's default [`Config`].
    ///
    /// # Errors
    /// If either controller or the planner refuses its configuration.
    pub fn new(scenario: &Scenario) -> Result<Self, SetupError> {
        Self::with_config(scenario, Config::default())
    }

    /// Build the system for a scenario with a given firmware configuration.
    ///
    /// # Errors
    /// If either controller or the planner refuses its configuration.
    pub fn with_config(scenario: &Scenario, config: Config) -> Result<Self, SetupError> {
        let config = Config {
            chest_sensing: config.chest_sensing && scenario.chest_sensing,
            ..config
        };
        let planner = BtePlanner::new(scenario.plant.waveform).map_err(SetupError::Waveform)?;
        let detector =
            CompressionDetector::new(aed_cpr::Settings::default()).map_err(SetupError::Chest)?;
        let rescue = Rescue::with_chest_sensor(config, Analyzer::new(), planner, detector)
            .map_err(SetupError::Config)?;
        let supervisor = Supervisor::new(config).map_err(SetupError::Config)?;
        let n = &scenario.noise;
        let mut synth = Synth::new(scenario.rhythm, scenario.seed);
        if n.rms_uv > 0.0 {
            synth = synth.with_noise(n.rms_uv);
        }
        if n.mains_uv > 0.0 {
            synth = synth.with_mains(n.mains_uv, 50.0);
        }
        if n.wander_uv > 0.0 {
            synth = synth.with_baseline_wander(n.wander_uv);
        }
        let mut plant = Plant::new(scenario.plant, synth, scenario.seed);
        plant.mode = scenario.mode;
        plant.after_shock.clone_from(&scenario.after_shock);
        plant.set_compressing(scenario.responder.cpr_at_start);
        plant.compression_uv = scenario.responder.compression_uv;
        plant.compression_per_min = scenario.responder.compression_per_min;
        plant.compression_ohms = scenario.responder.compression_ohms;
        plant.cardiac_ohms = scenario.cardiac_ohms;
        let plant = Rc::new(RefCell::new(plant));
        Ok(Simulation {
            scenario: scenario.clone(),
            main_view: MainView::new(Rc::clone(&plant)),
            sup_view: SupervisorView::new(Rc::clone(&plant)),
            plant,
            rescue,
            supervisor,
            hands_on_at_fire: Vec::new(),
            config,
            main_frozen: false,
            main_frozen_at: None,
            alarm_on: false,
            alarm_first_at: None,
            script_done: vec![false; scenario.script.len()],
            timeline: Vec::new(),
            states: vec![(0, RescueState::SelfTest)],
            decisions: Vec::new(),
            violations: Vec::new(),
            injected: Vec::new(),
            prompts_seen: 0,
            last_directive: None,
            silence_reported: false,
            main_events_seen: 0,
            sup_events_seen: 0,
            last_printed_decision: None,
            responder_pending: None,
            reattach_at: None,
            press_pads_at: None,
            pads_on_at: None,
            hands_off_since: if scenario.responder.cpr_at_start {
                None
            } else {
                Some(0)
            },
            cycle: None,
            timings: Vec::new(),
            charges_started: 0,
            refusals: Vec::new(),
            milestone_target: None,
            milestone_next: 0,
            cap_was_charged: false,
            overvoltage_reported: false,
        })
    }

    /// The plant (for inspection).
    #[must_use]
    pub fn plant(&self) -> &SharedPlant {
        &self.plant
    }

    /// The main controller's rescue sequence (for inspection).
    #[must_use]
    pub fn rescue(&self) -> &Rescue<Analyzer, BtePlanner, CompressionDetector> {
        &self.rescue
    }

    /// The supervisor (for inspection).
    #[must_use]
    pub fn supervisor(&self) -> &Supervisor {
        &self.supervisor
    }

    fn now(&self) -> u32 {
        self.plant.borrow().now_ms
    }

    fn log(&mut self, source: Source, text: impl Into<String>) {
        let t_ms = self.now();
        self.timeline.push(Entry {
            t_ms,
            source,
            text: text.into(),
        });
    }

    fn violate(&mut self, rule: Rule, detail: String, changed_before: Option<u32>) {
        let t_ms = self.now();
        self.log(Source::Violation, format!("{}: {detail}", rule.describe()));
        self.violations.push(Violation {
            t_ms,
            rule,
            detail,
            rhythm_changed_ms_before: changed_before,
        });
    }

    /// Run the whole scenario and return the result.
    #[must_use]
    pub fn run(mut self) -> RunResult {
        self.log(
            Source::Device,
            format!(
                "Switched on ({} mode). Self-test starts.",
                match self.scenario.mode {
                    PatientMode::Adult => "adult",
                    PatientMode::Child => "child",
                }
            ),
        );
        let r = self.plant.borrow().rhythm;
        self.log(Source::Patient, format!("Patient rhythm: {}", rhythm_name(r)));
        if self.scenario.responder.cpr_at_start {
            let (rate, ohms) = (self.scenario.responder.compression_per_min, self.scenario.responder.compression_ohms);
            self.log(
                Source::Responder,
                format!("Doing chest compressions ({rate:.0}/min, {ohms:.2} Ω each in the pad impedance)."),
            );
        }
        while self.now() < self.scenario.duration_ms {
            self.step();
        }
        self.finish()
    }

    /// Advance the whole system by 1 ms.
    pub fn step(&mut self) {
        let state_before = self.rescue.state();
        let events = self.plant.borrow_mut().step();
        let now = self.now();
        for e in events {
            self.on_plant_event(e, state_before);
        }

        if now.is_multiple_of(ECG_PERIOD_MS) {
            let sample = self.plant.borrow_mut().ecg_sample();
            if !self.main_frozen {
                let t = self.plant.borrow().main_clock();
                self.rescue.on_ecg_sample(sample, t);
            }
        }
        if now.is_multiple_of(IMPEDANCE_PERIOD_MS) {
            let sample = self.plant.borrow_mut().impedance_sample();
            if let (Some(sample), false) = (sample, self.main_frozen) {
                let t = self.plant.borrow().main_clock();
                self.rescue.on_impedance_sample(sample, t);
            }
        }
        self.plant.borrow_mut().update_hands_on();
        if now.is_multiple_of(MAIN_TICK_MS) && !self.main_frozen {
            self.rescue.tick(&mut self.main_view);
        }
        self.supervisor.tick(&mut self.sup_view);

        self.observe();
        self.run_script();
        self.run_responder();
    }

    fn on_plant_event(&mut self, e: PlantEvent, state_before: RescueState) {
        match e {
            PlantEvent::RhythmChanged(r) => {
                self.log(Source::Patient, format!("Rhythm changes to {}", rhythm_name(r)));
            }
            PlantEvent::BridgeDidNotFire => {
                self.log(
                    Source::Hardware,
                    "Interlock passed but the output bridge did not fire (injected fault).",
                );
            }
            PlantEvent::Shock(s) => self.on_shock(&s, state_before),
        }
    }

    fn on_shock(&mut self, s: &ShockRecord, state_before: RescueState) {
        let text = match s.result {
            Some(r) => format!(
                "SHOCK fired from {:.0} V into {:.0} Ω: {:.1} J delivered, peak {:.1} A, \
                 phases {:.2} + {:.2} + {:.2} ms, {:.0} V left on the capacitor (model)",
                s.capacitor_v,
                r.patient_ohms,
                r.patient_energy_j,
                r.peak_current_a,
                r.phase1_s * 1e3,
                r.interphase_s * 1e3,
                r.phase2_s * 1e3,
                r.final_capacitor_v
            ),
            None => format!(
                "SHOCK fired from {:.0} V into OPEN pads: no current flowed",
                s.capacitor_v
            ),
        };
        self.log(Source::Hardware, text);

        // Invariants at the moment the bridge fires (ground truth, not the analyser).
        if !rhythm_is_shockable(s.rhythm) {
            let before = s.at_ms.saturating_sub(s.rhythm_since_ms);
            self.violate(
                Rule::FiredOnNonShockableRhythm,
                format!(
                    "true rhythm was {} (for {} ms)",
                    rhythm_name(s.rhythm),
                    before
                ),
                Some(before),
            );
        }
        let in_range = s.patient_ohms.is_some_and(|r| {
            r >= f64::from(MIN_PATIENT_OHMS.0) && r <= f64::from(MAX_PATIENT_OHMS.0)
        });
        if !in_range {
            self.violate(
                Rule::FiredWithPadsOffOrOutOfRange,
                format!("true pad resistance {:?}", s.patient_ohms),
                None,
            );
        }
        if s.cancel_pressed {
            self.violate(Rule::FiredWhileCancelPressed, String::new(), None);
        }
        let hands_on_since = self.plant.borrow().hands_on_since;
        if let Some(since) = hands_on_since {
            let exposure = s.at_ms.saturating_sub(since);
            self.hands_on_at_fire.push(exposure);
            if self.config.chest_sensing && exposure > HANDS_ON_ALLOWANCE_MS {
                self.violate(
                    Rule::FiredWhileHandsOn,
                    format!("hands on the chest for {exposure} ms"),
                    None,
                );
            }
        }
        if state_before != RescueState::Delivering {
            self.violate(
                Rule::FiredOutsideDelivering,
                format!("main controller was in {state_before:?}"),
                None,
            );
        }

        // Timing.
        let number = self.plant.borrow().shocks.len();
        if let Some(c) = self.cycle.take() {
            let advised = c.advised.unwrap_or(c.analysis_start);
            let charge_start = c.charge_start.unwrap_or(advised);
            let charged = c.charged.unwrap_or(charge_start);
            let countdown_start = c.countdown_start.unwrap_or(advised);
            let request = c.request.unwrap_or(s.at_ms);
            self.timings.push(ShockTiming {
                number,
                at_ms: s.at_ms,
                analysis_ms: advised.saturating_sub(c.analysis_start),
                charge_ms: charged.saturating_sub(charge_start),
                charge_wait_ms: countdown_start.saturating_sub(advised),
                countdown_ms: request.saturating_sub(countdown_start),
                permit_wait_ms: s.at_ms.saturating_sub(request),
                hands_off_ms: self.hands_off_since.map(|t| s.at_ms.saturating_sub(t)),
            });
        }
    }

    /// Record what the controllers did this step.
    fn observe(&mut self) {
        let now = self.now();

        // State changes.
        let state = self.rescue.state();
        if self.states.last().map(|&(_, s)| s) != Some(state) {
            let from = self.states.last().map_or(RescueState::SelfTest, |&(_, s)| s);
            self.states.push((now, state));
            self.log(Source::Device, format!("State: {from:?} -> {state:?}"));
            match state {
                RescueState::Analysing | RescueState::AnalysingCharging => {
                    // The charge this analysis starts (D-29) is picked up
                    // from the event record just after this.
                    self.cycle = Some(Cycle {
                        analysis_start: now,
                        ..Cycle::default()
                    });
                    self.last_printed_decision = None;
                }
                RescueState::Charging => {
                    if let Some(c) = self.cycle.as_mut() {
                        c.advised.get_or_insert(now);
                    }
                }
                RescueState::Countdown => {
                    if let Some(c) = self.cycle.as_mut() {
                        c.advised.get_or_insert(now);
                        c.countdown_start = Some(now);
                    }
                }
                RescueState::Delivering => {
                    if let Some(c) = self.cycle.as_mut() {
                        c.request = Some(now);
                    }
                }
                _ => {}
            }
        }

        // Prompts.
        let new_prompts: Vec<(u32, Prompt)> = {
            let p = self.plant.borrow();
            p.prompts.get(self.prompts_seen..).unwrap_or_default().to_vec()
        };
        self.prompts_seen = self.prompts_seen.saturating_add(new_prompts.len());
        for (_, prompt) in new_prompts {
            self.log(Source::Said, format!("\"{}\"", prompt.text()));
            if !is_neutral(prompt) {
                self.last_directive = Some((self.now(), prompt));
                self.silence_reported = false;
            }
            self.responder_hears(prompt);
        }

        self.observe_main_events();
        self.observe_supervisor_events();
        self.observe_supervisor_alarm();
        self.observe_capacitor();
        self.check_not_silent();
    }

    /// The supervisor's own fault alarm: log it, and the responder treats it
    /// as "device fault, continue CPR" (what its label says).
    fn observe_supervisor_alarm(&mut self) {
        let on = self.plant.borrow().supervisor_alarm;
        if on == self.alarm_on {
            return;
        }
        self.alarm_on = on;
        if on {
            let now = self.now();
            self.alarm_first_at.get_or_insert(now);
            self.log(
                Source::Supervisor,
                "FAULT ALARM ON: the supervisor's own buzzer and light \
                 (\"Device fault. No shocks. Continue CPR.\")",
            );
            self.silence_reported = false;
            self.responder_wants(true);
        } else {
            self.log(Source::Supervisor, "Fault alarm off");
        }
    }

    /// The responder must never be left with only an informational prompt
    /// for longer than the grace time. The supervisor's fault alarm counts
    /// as an instruction ("continue CPR"). If the main controller freezes it
    /// cannot speak, so the supervisor's alarm must come on within the grace
    /// time instead.
    fn check_not_silent(&mut self) {
        if self.silence_reported || self.alarm_on {
            return;
        }
        let now = self.now();
        let grace = silence_grace_ms(&self.config);
        if let Some(frozen_at) = self.main_frozen_at {
            if now.saturating_sub(frozen_at) > grace {
                self.silence_reported = true;
                self.violate(
                    Rule::LeftWithoutInstruction,
                    format!("main controller frozen and no supervisor alarm within {grace} ms"),
                    None,
                );
            }
            return;
        }
        match self.last_directive {
            Some((t, p)) if !is_instruction(p) && now.saturating_sub(t) > grace => {
                self.silence_reported = true;
                self.violate(
                    Rule::LeftWithoutInstruction,
                    format!("last directive {p:?}, no instruction for {grace} ms"),
                    None,
                );
            }
            // A good self-test takes well under a second; silence beyond
            // twice the grace time after switch-on is a failure.
            None if now > grace.saturating_mul(2) => {
                self.silence_reported = true;
                self.violate(
                    Rule::SilentAtPowerOn,
                    format!("nothing said for {} ms after switch-on", grace.saturating_mul(2)),
                    None,
                );
            }
            _ => {}
        }
    }

    fn observe_main_events(&mut self) {
        let record = self.rescue.events();
        let total = record.len() as u64 + u64::from(record.overwritten());
        let new = total.saturating_sub(self.main_events_seen);
        self.main_events_seen = total;
        let skip = (record.len() as u64).saturating_sub(new);
        let events: Vec<Event> = record
            .iter()
            .skip(usize::try_from(skip).unwrap_or(0))
            .map(|e| e.event)
            .collect();
        for e in events {
            self.on_main_event(e);
        }
    }

    fn on_main_event(&mut self, e: Event) {
        let now = self.now();
        let text = match e {
            Event::Decision(d) => {
                self.decisions.push((now, d));
                if self.last_printed_decision == Some(d) {
                    return;
                }
                self.last_printed_decision = Some(d);
                self.log(
                    Source::Analysis,
                    format!(
                        "{} (repeats each second until it changes)",
                        decision_text(d)
                    ),
                );
                return;
            }
            Event::AnalysisTimedOut => {
                "No answer from the analysis within its time limit: no shock".to_string()
            }
            Event::AnalysisInconclusive => {
                "Analysis inconclusive ('not sure' until the time limit): no shock".to_string()
            }
            Event::PadsOn => "Pad contact detected".to_string(),
            Event::PadsOff => "Pad contact lost".to_string(),
            Event::PadResistance(o) => format!("Pad resistance measured just before shock: {} Ω", o.0),
            Event::PatientModeChanged(m) => format!("Patient mode switched to {m:?}"),
            Event::ChargeStarted(v) => {
                self.charges_started = self.charges_started.saturating_add(1);
                match self.cycle.as_mut() {
                    Some(c) => c.charge_start = Some(now),
                    None => {
                        self.cycle = Some(Cycle {
                            charge_start: Some(now),
                            ..Cycle::default()
                        });
                    }
                }
                format!("Charging to {} V", v.0)
            }
            Event::Charged(v) => {
                if let Some(c) = self.cycle.as_mut() {
                    c.charged = Some(now);
                }
                format!("Charged ({} V on its own sensor)", v.0)
            }
            Event::ChargingDuringAnalysis => {
                "Analysing AND charging (D-29): the capacitor charges in case a shock is advised"
                    .to_string()
            }
            Event::EmptyingStarted => {
                "Lets go of the capacitor: emptying in the background into the internal load"
                    .to_string()
            }
            Event::ChargeBurnedOff => "Unused charge burned off (no shock)".to_string(),
            Event::PadContactPoor(o) => {
                format!("Pads on but {} Ω: above the 175 Ω shock range (poor contact)", o.0)
            }
            Event::CancelIgnoredDuringSelfTest => {
                "Cancel pressed during the self-test: ignored (D-31)".to_string()
            }
            Event::CancelStuck => "Cancel control STUCK (held too long)".to_string(),
            Event::Plan(p) => format!(
                "Shock plan: {:.2} ms + {:.2} ms gap + {:.2} ms, expects {:.1} J",
                f64::from(p.phase1.0) / 1e3,
                f64::from(p.interphase.0) / 1e3,
                f64::from(p.phase2.0) / 1e3,
                f64::from(p.expected_energy.0) / 10.0
            ),
            Event::PlanRefused(e) => format!("Planner refused: {e:?}"),
            Event::ShockRequested => "Shock request raised; waiting for supervisor permit".to_string(),
            Event::Shock(o) => match o {
                ShockOutcome::Delivered => "Output stage reports the shock delivered".to_string(),
                ShockOutcome::RefusedByInterlock => "Interlock refused the shock".to_string(),
                ShockOutcome::TimedOut => {
                    "No shock within the delivery time limit: request withdrawn".to_string()
                }
                ShockOutcome::Fault => "Output stage reported a fault".to_string(),
                ShockOutcome::Withdrawn(r) => format!("Shock request withdrawn: {}", abort_text(r)),
            },
            Event::ShockAborted(r) => format!("Shock abandoned: {}", abort_text(r)),
            Event::CapacitorEmptied(v) => {
                format!("Capacitor confirmed empty ({} V on its own sensor)", v.0)
            }
            Event::Fault(k) => format!("FAULT: {k:?}"),
            Event::CancelPressed => "Cancel pressed".to_string(),
            Event::BatteryLow => "Battery low".to_string(),
            Event::SelfTestPassed => "Self-test passed".to_string(),
            Event::SelfTestFailed(f) => format!("Self-test failed: {f:?}"),
            Event::AnalysisHeld(s) => format!(
                "Chest {}: analysis reset, waiting for hands off",
                match s {
                    aed_types::ChestState::Compressing => "being compressed",
                    _ => "not still (cannot tell)",
                }
            ),
            Event::ChestSensingFailed(s) => {
                format!("Compression sensing FAILED ({s:?}): heart signal only from now on")
            }
            Event::ChestSensingRestored => "Compression sensing working again".to_string(),
            Event::CprPause(ms) => format!("CPR pause of {:.1} s", f64::from(ms) / 1000.0),
            Event::CprPeriodSummary {
                compressions,
                rate_per_min,
                on_chest_permille,
                longest_pause_ms,
            } => format!(
                "CPR period: {compressions} compressions, {rate_per_min}/min, {:.0}% of the time on the chest, longest pause {:.1} s",
                f64::from(on_chest_permille) / 10.0,
                f64::from(longest_pause_ms) / 1000.0
            ),
            Event::NoCompressionsPrompt => "No compressions sensed: prompts to continue CPR".to_string(),
            // The supervisor's own record covers these, and state changes are polled.
            Event::SupervisorPermitted
            | Event::SupervisorRefused(_)
            | Event::SupervisorReleasedHold(_)
            | Event::SupervisorFaultLatched(_)
            | Event::StateChanged { .. }
            | Event::PowerOn => return,
        };
        self.log(Source::Device, text);
    }

    fn observe_supervisor_events(&mut self) {
        let record = self.supervisor.events();
        let total = record.len() as u64 + u64::from(record.overwritten());
        let new = total.saturating_sub(self.sup_events_seen);
        self.sup_events_seen = total;
        let skip = (record.len() as u64).saturating_sub(new);
        let events: Vec<Event> = record
            .iter()
            .skip(usize::try_from(skip).unwrap_or(0))
            .map(|e| e.event)
            .collect();
        for e in events {
            let text = match e {
                Event::SupervisorPermitted => "Permit given (independent checks passed)".to_string(),
                Event::SupervisorRefused(r) => {
                    if !self.refusals.contains(&r) {
                        self.refusals.push(r);
                    }
                    format!("Permit refused: {}", refusal_text(r))
                }
                Event::SupervisorReleasedHold(r) => match r {
                    ReleaseReason::MainNotArmed => {
                        "Lets go of the discharge path (main no longer armed)".to_string()
                    }
                    ReleaseReason::HeartbeatStale => {
                        "Lets go of the discharge path: main controller's heartbeat stopped"
                            .to_string()
                    }
                    ReleaseReason::Cancel => {
                        "Lets go of the discharge path: cancel pressed".to_string()
                    }
                    ReleaseReason::Fault(f) => {
                        format!("Lets go of the discharge path: fault {f:?}")
                    }
                },
                Event::SupervisorFaultLatched(f) => format!("FAULT latched: {f:?}"),
                _ => continue,
            };
            self.log(Source::Supervisor, text);
        }
    }

    fn observe_capacitor(&mut self) {
        let (v, target, limit) = {
            let p = self.plant.borrow();
            (p.cap_v, p.charger_target, f64::from(self.config.overvoltage_limit.0))
        };
        let empty = f64::from(self.config.empty_voltage.0);
        // Charge-curve milestones: 25/50/75/100 % of the charger's target.
        let target_v = target.map(|t| t.0);
        if target_v != self.milestone_target {
            self.milestone_target = target_v;
            self.milestone_next = 1;
        }
        if let Some(t) = target_v {
            if self.milestone_next <= 4 {
                let level = f64::from(t) * f64::from(self.milestone_next) / 4.0;
                if v >= level - 0.5 {
                    self.log(
                        Source::Hardware,
                        format!(
                            "Capacitor at {:.0} V ({}% of the {} V target)",
                            v,
                            u32::from(self.milestone_next) * 25,
                            t
                        ),
                    );
                    self.milestone_next = self.milestone_next.saturating_add(1);
                }
            }
        }
        if v >= empty {
            self.cap_was_charged = true;
        } else if self.cap_was_charged {
            self.cap_was_charged = false;
            self.log(
                Source::Hardware,
                format!("Capacitor below {empty:.0} V (true voltage)"),
            );
        }
        if v > limit + OVERVOLTAGE_MARGIN_V && !self.overvoltage_reported {
            self.overvoltage_reported = true;
            self.violate(
                Rule::Overvoltage,
                format!("true voltage {v:.0} V, limit {limit:.0} V"),
                None,
            );
        }
    }

    fn trigger_due(&self, trigger: Trigger) -> bool {
        let now = self.now();
        match trigger {
            Trigger::At(t) => now >= t,
            Trigger::AfterState {
                state,
                nth,
                after_ms,
            } => self
                .states
                .iter()
                .filter(|&&(_, s)| s == state)
                .nth(usize::try_from(nth.saturating_sub(1)).unwrap_or(0))
                .is_some_and(|&(t, _)| now >= t.saturating_add(after_ms)),
        }
    }

    fn run_script(&mut self) {
        let now = self.now();
        if self.pads_on_at.is_none() && now >= self.scenario.pads_on_at_ms {
            self.pads_on_at = Some(now);
            let ohms = self.scenario.pad_ohms;
            // A "pads fall off" fault injected earlier wins: they never stick.
            if !self.plant.borrow().has_fault(Fault::PadsOff) {
                self.plant.borrow_mut().pad_ohms = Some(ohms);
                self.log(Source::Responder, format!("Pads attached ({ohms:.0} Ω)."));
            }
        }
        if let Some(at) = self.press_pads_at {
            if now >= at {
                self.press_pads_at = None;
                if let Some(ohms) = self.scenario.responder.pressing_pads_brings_ohms_to {
                    if self.plant.borrow().pad_ohms.is_some() {
                        self.plant.borrow_mut().pad_ohms = Some(ohms);
                        self.log(
                            Source::Responder,
                            format!("Presses the pads down firmly: now {ohms:.0} Ω."),
                        );
                    }
                }
            }
        }
        if let Some(at) = self.reattach_at {
            if now >= at {
                self.reattach_at = None;
                let ohms = self.scenario.pad_ohms;
                self.plant.borrow_mut().pad_ohms = Some(ohms);
                self.log(Source::Responder, format!("Pads re-attached ({ohms:.0} Ω)."));
            }
        }
        for i in 0..self.scenario.script.len() {
            if self.script_done.get(i).copied().unwrap_or(true) {
                continue;
            }
            let Some(&(trigger, action)) = self.scenario.script.get(i) else {
                continue;
            };
            if !self.trigger_due(trigger) {
                continue;
            }
            if let Some(done) = self.script_done.get_mut(i) {
                *done = true;
            }
            self.apply(action);
        }
    }

    fn apply(&mut self, action: Action) {
        let now = self.now();
        match action {
            Action::Fault(f) => {
                self.plant.borrow_mut().inject(f);
                if f == Fault::MainFreeze && !self.main_frozen {
                    self.main_frozen = true;
                    self.main_frozen_at = Some(now);
                }
                self.injected.push((now, f));
                self.log(Source::Injected, format!("Fault: {}", f.describe()));
            }
            Action::SetRhythm(r) => self.plant.borrow_mut().set_rhythm(r),
            Action::SetPadOhms(o) => {
                self.plant.borrow_mut().pad_ohms = o;
                match o {
                    Some(r) => self.log(Source::Patient, format!("Pad resistance now {r:.0} Ω")),
                    None => self.log(Source::Patient, "Pads come off"),
                }
            }
            Action::PressCancel(ms) => {
                self.plant.borrow_mut().press_cancel(ms);
                self.log(Source::Responder, format!("Presses cancel ({ms} ms)"));
            }
            Action::SetMode(m) => {
                self.plant.borrow_mut().mode = m;
                self.log(Source::Responder, format!("Moves the switch to {m:?}"));
            }
            Action::Touch(ms) => {
                self.plant.borrow_mut().touch(ms);
                self.log(Source::Responder, format!("Presses on the chest ({ms} ms)"));
            }
            Action::Ventilate(on) => {
                self.plant.borrow_mut().ventilating = on;
                self.log(
                    Source::Responder,
                    if on {
                        "Starts giving breaths every 6 s."
                    } else {
                        "Stops giving breaths."
                    },
                );
            }
        }
    }

    fn responder_hears(&mut self, prompt: Prompt) {
        let now = self.now();
        let want = match prompt {
            Prompt::StartCpr
            | Prompt::ContinueCpr
            | Prompt::DeviceFaultContinueCpr
            | Prompt::SeriousFaultContinueCpr
            | Prompt::CancelStuckContinueCpr => Some(true),
            Prompt::AnalysingDoNotTouch
            | Prompt::StopCprAnalysing
            | Prompt::DoNotTouchPatient
            | Prompt::ShockAdvised
            | Prompt::ShockAdvisedCharging
            | Prompt::StandClear => Some(false),
            Prompt::PressPadsFirmly => {
                if self.scenario.responder.pressing_pads_brings_ohms_to.is_some()
                    && self.press_pads_at.is_none()
                {
                    self.press_pads_at = Some(now.saturating_add(self.scenario.responder.reaction_ms));
                }
                None
            }
            Prompt::AttachPads => {
                let off = self.plant.borrow().pad_ohms.is_none();
                if off && self.pads_on_at.is_some() && self.reattach_at.is_none() {
                    if let Some(d) = self.scenario.responder.reattach_after_ms {
                        self.reattach_at = Some(now.saturating_add(d));
                    }
                }
                None
            }
            _ => None,
        };
        if let Some(w) = want {
            self.responder_wants(w);
        }
    }

    /// The responder decides to start (`true`) or stop (`false`)
    /// compressions, and does so after their reaction time.
    fn responder_wants(&mut self, w: bool) {
        if !w && self.scenario.responder.keeps_compressing {
            return;
        }
        let now = self.now();
        let reaction = self.scenario.responder.reaction_ms;
        let current = self.responder_pending.map_or_else(
            || self.plant.borrow().compressing,
            |(_, pending)| pending,
        );
        if current != w {
            self.responder_pending = Some((now.saturating_add(reaction), w));
        }
    }

    fn run_responder(&mut self) {
        let now = self.now();
        if let Some((at, w)) = self.responder_pending {
            if now >= at {
                self.responder_pending = None;
                let changed = {
                    let mut p = self.plant.borrow_mut();
                    let changed = p.compressing != w;
                    p.set_compressing(w);
                    changed
                };
                if changed {
                    if w {
                        self.hands_off_since = None;
                        self.log(Source::Responder, "Starts chest compressions.");
                    } else {
                        self.hands_off_since = Some(now);
                        self.log(Source::Responder, "Stops compressions, hands off.");
                    }
                }
            }
        }
    }

    fn finish(mut self) -> RunResult {
        let (final_v, max_v, faults, prompts, shocks, beats, dropped) = {
            let p = self.plant.borrow();
            (
                p.cap_v,
                p.max_cap_v,
                p.faults.clone(),
                p.prompts.clone(),
                p.shocks.clone(),
                p.metronome_beats,
                p.messages_dropped,
            )
        };
        let empty = f64::from(self.config.empty_voltage.0);
        let state = self.rescue.state();
        let stuck = faults.contains(&Fault::DischargeStuck);
        let frozen = faults.contains(&Fault::MainFreeze);
        // The Fault state is entered at once and the capacitor empties in
        // the background (the main controller checks it against its own
        // time limit), so a run that ends within that time is not a failure.
        if state == RescueState::Fault && final_v >= empty && !stuck && !self.rescue.emptying() {
            self.violate(
                Rule::FaultLeftCharged,
                format!("{final_v:.0} V at the end"),
                None,
            );
        }
        // Charged on purpose (or still emptying in the background, which
        // the main controller checks against its own time limit).
        let armed_now = !frozen
            && (state == RescueState::SelfTest
                || state.holds_charge()
                || self.rescue.emptying());
        if stuck
            && !armed_now
            && final_v >= empty
            && self.rescue.fault() != Some(FaultKind::CapacitorNotEmptied)
        {
            self.violate(
                Rule::StuckDischargeNotReportedSerious,
                format!("{final_v:.0} V, fault {:?}", self.rescue.fault()),
                None,
            );
        }
        if final_v >= empty && !armed_now && !stuck {
            self.violate(
                Rule::CapacitorLeftCharged,
                format!("{final_v:.0} V at the end, main controller in {state:?}"),
                None,
            );
        }
        let charges_burned = self.rescue.counters().charges_burned;
        let (charger_energy_j, dump_energy_j) = {
            let p = self.plant.borrow();
            (p.charger_energy_j, p.dump_energy_j)
        };
        RunResult {
            charges_burned,
            charger_energy_j,
            dump_energy_j,
            scenario: self.scenario,
            timeline: self.timeline,
            prompts,
            states: self.states,
            decisions: self.decisions,
            shocks,
            timings: self.timings,
            violations: self.violations,
            final_state: state,
            final_fault: self.rescue.fault(),
            supervisor_fault: self.supervisor.fault(),
            final_cap_v: final_v,
            max_cap_v: max_v,
            injected: self.injected,
            pads_on_at_ms: self.pads_on_at,
            charges_started: self.charges_started,
            supervisor_refusals: self.refusals,
            metronome_beats: beats,
            messages_dropped: dropped,
            supervisor_alarm_at: self.alarm_first_at,
            main_frozen_at: self.main_frozen_at,
            config: self.config,
            hands_on_at_fire: self.hands_on_at_fire,
            compressions_counted: self.rescue.counters().compressions,
            hands_on_aborts: self.rescue.counters().hands_on_aborts,
        }
    }
}

/// Run a scenario with the default firmware configuration.
///
/// # Errors
/// If a configuration is refused.
pub fn run(scenario: &Scenario) -> Result<RunResult, SetupError> {
    Ok(Simulation::new(scenario)?.run())
}

/// Plain English for a rhythm decision.
#[must_use]
pub const fn decision_text(d: RhythmDecision) -> &'static str {
    match d {
        RhythmDecision::Shockable => "Analysis says: SHOCKABLE",
        RhythmDecision::NonShockable => "Analysis says: not shockable",
        RhythmDecision::Indeterminate => "Analysis says: not sure",
    }
}

/// Plain English for an abort reason.
#[must_use]
pub fn abort_text(r: AbortReason) -> String {
    match r {
        AbortReason::Cancelled => "cancel pressed".to_string(),
        AbortReason::RhythmChanged => "rhythm no longer shockable".to_string(),
        AbortReason::PadsOff => "pads off".to_string(),
        AbortReason::ModeChanged => "adult/child switch moved".to_string(),
        AbortReason::PlanRefused(e) => format!("no shock plan ({e:?})"),
        AbortReason::ConfirmationStale => "rhythm confirmation too old".to_string(),
        AbortReason::HandsOnChest => "chest compressed or moving after the advice".to_string(),
    }
}

/// Plain English for a permit refusal.
#[must_use]
pub const fn refusal_text(r: PermitRefusal) -> &'static str {
    match r {
        PermitRefusal::Faulted => "a fault is latched",
        PermitRefusal::HeartbeatStale => "main controller's heartbeat is stale",
        PermitRefusal::AlreadyUsed => "permit already used for this charge",
        PermitRefusal::CancelPressed => "cancel pressed",
        PermitRefusal::CountdownTooShort => "countdown not yet complete on the supervisor's clock",
        PermitRefusal::NotHolding => "supervisor not holding the charge",
        PermitRefusal::Overvoltage => "overvoltage",
        PermitRefusal::VoltageDisagreesWithMain => "its voltage reading disagrees with main's",
        PermitRefusal::VoltageNotAtPlan => "voltage not at the planned charge",
        PermitRefusal::PadsOutOfRange => "its pad reading is out of range",
    }
}
