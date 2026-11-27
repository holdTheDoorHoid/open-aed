// SPDX-License-Identifier: GPL-3.0-or-later
//! Scenarios: who the patient is, what their heart does, what the responder
//! does, and what goes wrong, and when.
//!
//! [`catalogue`] holds the named scenarios the command-line program and the
//! integration tests run.

use aed_core::RescueState;
use aed_rhythm::synth::Rhythm;
use aed_types::PatientMode;

use crate::plant::{Fault, PlantConfig};

/// When a scripted action happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// At this simulation time, ms.
    At(u32),
    /// `after_ms` after the device enters `state` for the `nth` time (1 = first).
    AfterState {
        /// The state.
        state: RescueState,
        /// Which entry into it (1 is the first).
        nth: u32,
        /// Delay after entering it, ms.
        after_ms: u32,
    },
}

/// Something the script does to the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// Switch on a fault.
    Fault(Fault),
    /// Change the patient's rhythm.
    SetRhythm(Rhythm),
    /// Change the pad resistance (`None`: pads off).
    SetPadOhms(Option<f64>),
    /// Hold the cancel control for this many ms.
    PressCancel(u32),
    /// Move the adult/child switch.
    SetMode(PatientMode),
}

/// How the person doing the rescue behaves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Responder {
    /// Compressions are under way when the device is switched on.
    pub cpr_at_start: bool,
    /// Time to stop or start compressions after being told to, ms. Default
    /// 700 ms (model choice).
    pub reaction_ms: u32,
    /// Size of the compression artifact in the heart signal, µV peak to peak.
    /// Default 1,500 µV (model choice).
    pub compression_uv: f64,
    /// If the pads come off and the device says to attach them, the responder
    /// puts them back after this long (ms). `None`: never.
    pub reattach_after_ms: Option<u32>,
    /// The responder never stops compressions, whatever the device says
    /// (models someone who cannot hear or does not follow the device).
    /// Default false.
    pub keeps_compressing: bool,
}

impl Default for Responder {
    fn default() -> Self {
        Responder {
            cpr_at_start: true,
            reaction_ms: 700,
            compression_uv: 1_500.0,
            reattach_after_ms: None,
            keeps_compressing: false,
        }
    }
}

/// Extra interference on the heart signal.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Noise {
    /// Random noise, µV RMS.
    pub rms_uv: f32,
    /// Mains hum at 50 Hz, µV peak.
    pub mains_uv: f32,
    /// Slow baseline wander, µV peak.
    pub wander_uv: f32,
}

/// One scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    /// Command-line name.
    pub name: String,
    /// One-line plain-English description.
    pub description: String,
    /// Adult or child setting on the switch.
    pub mode: PatientMode,
    /// The patient's pad resistance once the pads are on, ohms.
    pub pad_ohms: f64,
    /// When the pads go on, ms after switch-on.
    pub pads_on_at_ms: u32,
    /// The rhythm at the start.
    pub rhythm: Rhythm,
    /// What the rhythm becomes after the 1st, 2nd, ... shock (`None`: unchanged).
    pub after_shock: Vec<Option<Rhythm>>,
    /// Interference on the heart signal.
    pub noise: Noise,
    /// Scripted actions.
    pub script: Vec<(Trigger, Action)>,
    /// The responder.
    pub responder: Responder,
    /// The device's physics.
    pub plant: PlantConfig,
    /// How long to run, ms.
    pub duration_ms: u32,
    /// Seed for every random choice (signal and wire).
    pub seed: u32,
    /// What the test checks about this scenario.
    pub expect: Expect,
}

/// The safety outcome a scenario must show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// Exactly this many shocks, and no fault.
    Shocks(usize),
    /// No shock and the capacitor is never charged for a shock.
    NeverCharges,
    /// A shock was advised and prepared, but abandoned: no shock, capacitor emptied.
    AbortedNoShock,
    /// No shock at all; the device ends in the Fault state.
    FaultNoShock,
    /// The device ends in the Fault state (shocks before the fault allowed).
    EndsInFault,
    /// No shock (anything else is reported, not asserted).
    NoShock,
    /// Nothing particular beyond the invariants.
    InvariantsOnly,
}

impl Scenario {
    /// A base scenario: adult, 80 Ω, coarse VF that converts to normal rhythm
    /// after the first shock, pads on 8 s after switch-on, 170 s long.
    #[must_use]
    pub fn base(name: &str, description: &str) -> Self {
        Scenario {
            name: name.to_string(),
            description: description.to_string(),
            mode: PatientMode::Adult,
            pad_ohms: 80.0,
            pads_on_at_ms: 8_000,
            rhythm: Rhythm::coarse_vf(),
            after_shock: vec![Some(Rhythm::sinus(75.0))],
            noise: Noise::default(),
            script: Vec::new(),
            responder: Responder::default(),
            plant: PlantConfig::default(),
            duration_ms: 170_000,
            seed: 1,
            expect: Expect::Shocks(1),
        }
    }

    fn with(mut self, f: impl FnOnce(&mut Scenario)) -> Self {
        f(&mut self);
        self
    }

    /// The fault scenario's injected fault, if it has exactly one.
    #[must_use]
    pub fn injected_faults(&self) -> Vec<Fault> {
        self.script
            .iter()
            .filter_map(|(_, a)| match a {
                Action::Fault(f) => Some(*f),
                _ => None,
            })
            .collect()
    }
}

const fn after(state: RescueState, after_ms: u32) -> Trigger {
    Trigger::AfterState {
        state,
        nth: 1,
        after_ms,
    }
}

/// A scenario that injects one fault at the given moment.
fn fault_scenario(fault: Fault, when: Trigger, expect: Expect, note: &str) -> Scenario {
    let name = format!("fault-{}", fault.slug());
    let description = format!("Adult VF; {} ({note}).", fault.describe());
    Scenario::base(&name, &description).with(|s| {
        s.script.push((when, Action::Fault(fault)));
        s.expect = expect;
        s.duration_ms = 90_000;
    })
}

/// Every named scenario.
#[must_use]
pub fn catalogue() -> Vec<Scenario> {
    use RescueState::{Analysing, Charging, Countdown};
    let sinus = Rhythm::sinus(75.0);
    let mut v = vec![
        Scenario::base(
            "adult-vf",
            "Adult in coarse VF at 80 ohms; one shock converts it to a normal rhythm.",
        ),
        Scenario::base(
            "adult-vf-two-shocks",
            "Adult in coarse VF; the first shock fails, the second converts it.",
        )
        .with(|s| {
            s.after_shock = vec![None, Some(sinus)];
            s.duration_ms = 320_000;
            s.expect = Expect::Shocks(2);
        }),
        Scenario::base(
            "child-vf",
            "Child in coarse VF at 60 ohms, child mode; one shock converts it.",
        )
        .with(|s| {
            s.mode = PatientMode::Child;
            s.pad_ohms = 60.0;
        }),
        Scenario::base("normal-rhythm", "Normal rhythm throughout: must never charge.").with(
            |s| {
                s.rhythm = Rhythm::sinus(80.0);
                s.after_shock = Vec::new();
                s.expect = Expect::NeverCharges;
            },
        ),
        Scenario::base("asystole", "Flatline throughout: no shock, CPR.").with(|s| {
            s.rhythm = Rhythm::Asystole;
            s.after_shock = Vec::new();
            s.expect = Expect::NeverCharges;
        }),
        Scenario::base(
            "vf-converts-during-countdown",
            "VF turns into a normal rhythm 0.5 s into the countdown: must abort and empty.",
        )
        .with(|s| {
            s.script
                .push((after(Countdown, 500), Action::SetRhythm(sinus)));
            s.expect = Expect::AbortedNoShock;
            s.duration_ms = 90_000;
        }),
        Scenario::base(
            "cancel-during-countdown",
            "The responder presses cancel 1.5 s into the countdown.",
        )
        .with(|s| {
            s.script
                .push((after(Countdown, 1_500), Action::PressCancel(800)));
            s.expect = Expect::AbortedNoShock;
            s.duration_ms = 90_000;
        }),
        Scenario::base(
            "pads-off-during-charge",
            "A pad comes off 2 s into charging; the responder re-attaches it 8 s after being told.",
        )
        .with(|s| {
            s.script
                .push((after(Charging, 2_000), Action::SetPadOhms(None)));
            s.responder.reattach_after_ms = Some(8_000);
            s.expect = Expect::Shocks(1);
        }),
        Scenario::base("high-resistance", "Adult VF at 175 ohms (top of the range).")
            .with(|s| s.pad_ohms = 175.0),
        Scenario::base("low-resistance", "Adult VF at 25 ohms (bottom of the range).")
            .with(|s| s.pad_ohms = 25.0),
        Scenario::base(
            "noisy",
            "Adult VF with random noise, mains hum and baseline wander on the signal.",
        )
        .with(|s| {
            s.noise = Noise {
                rms_uv: 40.0,
                mains_uv: 150.0,
                wander_uv: 300.0,
            };
            s.expect = Expect::InvariantsOnly;
        }),
        Scenario::base("rapid-vt", "Adult in rapid VT at 180/min; one shock converts it.")
            .with(|s| s.rhythm = Rhythm::rapid_vt(180.0)),
        Scenario::base(
            "fine-vf",
            "Adult in fine VF (120 µV): the analyser is designed to say 'not sure'.",
        )
        .with(|s| {
            s.rhythm = Rhythm::fine_vf();
            s.expect = Expect::NoShock;
        }),
        Scenario::base(
            "slow-hands-off",
            "Adult VF; the responder takes 2.5 s to stop compressions when told.",
        )
        .with(|s| {
            s.responder.reaction_ms = 2_500;
        }),
        Scenario::base(
            "never-stops-compressing",
            "Adult VF; the responder never stops compressions: no shock, 'unable to analyse', CPR.",
        )
        .with(|s| {
            s.responder.keeps_compressing = true;
            s.expect = Expect::NeverCharges;
        }),
        Scenario::base(
            "clock-drift",
            "Adult VF; supervisor clock 2% slow, main clock 2% fast.",
        )
        .with(|s| {
            s.plant.main_clock_ppm = 20_000;
            s.plant.supervisor_clock_ppm = -20_000;
        }),
        Scenario::base(
            "slow-link",
            "Adult VF; each message between the controllers takes 40 ms and 5% are lost.",
        )
        .with(|s| {
            s.plant.link_delay_ms = 40;
            s.plant.link_drop_probability = 0.05;
            s.expect = Expect::InvariantsOnly;
        }),
        Scenario::base(
            "sensor-tolerance",
            "Adult VF; main voltage sensor reads +1%, supervisor's -1% (small, opposite errors).",
        )
        .with(|s| {
            s.plant.main_voltage_sensor.gain = 1.01;
            s.plant.supervisor_voltage_sensor.gain = 0.99;
        }),
    ];

    v.push(fault_scenario(
        Fault::MainVoltageSensorWrong,
        after(Charging, 1_000),
        Expect::FaultNoShock,
        "during charging",
    ));
    v.push(fault_scenario(
        Fault::SupervisorVoltageSensorWrong,
        after(Charging, 1_000),
        Expect::FaultNoShock,
        "during charging",
    ));
    v.push(fault_scenario(
        Fault::ChargerWeak,
        after(RescueState::AttachPads, 0),
        Expect::FaultNoShock,
        "from just after the self-test",
    ));
    v.push(fault_scenario(
        Fault::ChargerRunaway,
        after(Charging, 500),
        Expect::FaultNoShock,
        "during charging",
    ));
    v.push(fault_scenario(
        Fault::DischargeStuck,
        after(Countdown, 0),
        Expect::EndsInFault,
        "from the start of the countdown; the shock still goes",
    ));
    v.push(fault_scenario(
        Fault::MainFreeze,
        after(Countdown, 2_000),
        Expect::NoShock,
        "2 s into the countdown",
    ));
    v.push(fault_scenario(
        Fault::LinkDrop,
        after(Charging, 1_000),
        Expect::FaultNoShock,
        "during charging",
    ));
    v.push(fault_scenario(
        Fault::PadsOff,
        after(Countdown, 2_000),
        Expect::AbortedNoShock,
        "2 s into the countdown; nobody puts them back",
    ));
    v.push(fault_scenario(
        Fault::CancelPressed,
        after(Analysing, 2_000),
        Expect::NoShock,
        "2 s into the first analysis",
    ));
    v.push(fault_scenario(
        Fault::BatteryDepleted,
        after(Charging, 1_000),
        Expect::FaultNoShock,
        "during charging",
    ));
    v.push(fault_scenario(
        Fault::BridgeFailsToFire,
        Trigger::At(0),
        Expect::NoShock,
        "from switch-on",
    ));
    v
}

/// The named scenario, if it exists.
#[must_use]
pub fn by_name(name: &str) -> Option<Scenario> {
    catalogue().into_iter().find(|s| s.name == name)
}
