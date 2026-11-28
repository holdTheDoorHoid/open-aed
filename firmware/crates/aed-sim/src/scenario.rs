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
    /// Someone presses on the chest (a touch, not compressions) for this many ms.
    Touch(u32),
    /// Someone starts (`true`) or stops giving breaths every 6 s.
    Ventilate(bool),
    /// The surroundings change to this temperature, °C.
    SetAmbient(f64),
    /// A temperature sensor fails (or is repaired).
    TempSensor(aed_core::TempPoint, crate::thermal::TempSensorFault),
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
    /// When told "Press pads firmly", the responder does so (after their
    /// reaction time) and the pad resistance becomes this, ohms. `None`
    /// (the default): pressing makes no difference.
    pub pressing_pads_brings_ohms_to: Option<f64>,
    /// Compression rate, per minute. Default 110.
    pub compression_per_min: f64,
    /// Change in pad impedance from each compression, ohms peak to peak.
    /// Default 1.5 Ω (see `Plant::compression_ohms`).
    pub compression_ohms: f64,
}

impl Default for Responder {
    fn default() -> Self {
        Responder {
            cpr_at_start: true,
            reaction_ms: 700,
            compression_uv: 1_500.0,
            reattach_after_ms: None,
            keeps_compressing: false,
            pressing_pads_brings_ohms_to: None,
            compression_per_min: 110.0,
            compression_ohms: 1.5,
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
    /// The firmware uses compression sensing (aed-core
    /// `Config::chest_sensing`). Default true; false gives the earlier,
    /// heart-signal-only behaviour for comparison.
    pub chest_sensing: bool,
    /// Change in pad impedance from each heartbeat of a beating heart,
    /// ohms (see `Plant::cardiac_ohms`). Default 0.05 Ω.
    pub cardiac_ohms: f64,
    /// The surroundings, °C. Default 20 °C.
    pub ambient_c: f64,
    /// The inside of the device at switch-on, °C. Default 20 °C.
    pub interior_c: f64,
}

/// The safety outcome a scenario must show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// Exactly this many shocks, and no fault.
    Shocks(usize),
    /// No shock is ever advised: no countdown, no shock request, no shock.
    /// (With charging during analysis, D-29, the capacitor does charge
    /// during each analysis; it must never go above the shock voltage.)
    NeverCountsDown,
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
            chest_sensing: true,
            cardiac_ohms: 0.05,
            ambient_c: 20.0,
            interior_c: 20.0,
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
    use RescueState::{AnalysingCharging, Countdown};
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
                s.expect = Expect::NeverCountsDown;
            },
        ),
        Scenario::base("asystole", "Flatline throughout: no shock, CPR.").with(|s| {
            s.rhythm = Rhythm::Asystole;
            s.after_shock = Vec::new();
            s.expect = Expect::NeverCountsDown;
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
            "A pad comes off 3 s into the analysis, while charging; the responder re-attaches it 8 s after being told.",
        )
        .with(|s| {
            s.script
                .push((after(AnalysingCharging, 3_000), Action::SetPadOhms(None)));
            s.responder.reattach_after_ms = Some(8_000);
            s.expect = Expect::Shocks(1);
        }),
        Scenario::base(
            "pads-off-during-countdown",
            "A pad comes off 1 s into the countdown; the responder re-attaches it 5 s after being told.",
        )
        .with(|s| {
            s.script
                .push((after(Countdown, 1_000), Action::SetPadOhms(None)));
            s.responder.reattach_after_ms = Some(5_000);
            s.expect = Expect::Shocks(1);
        }),
        Scenario::base(
            "pads-190-throughout",
            "Adult VF; pads on but reading 190 ohms (above the 175 ohm shock range) throughout: analyses, 'press pads firmly', no shock, CPR.",
        )
        .with(|s| {
            s.pad_ohms = 190.0;
            s.after_shock = Vec::new();
            s.expect = Expect::NoShock;
        }),
        Scenario::base(
            "pads-190-pressed-to-150",
            "Adult VF; pads on at 190 ohms; when told to press them firmly the responder does, and they read 150 ohms: shock.",
        )
        .with(|s| {
            s.pad_ohms = 190.0;
            s.responder.pressing_pads_brings_ohms_to = Some(150.0);
        }),
        Scenario::base(
            "stray-cancel-at-switch-on",
            "Adult VF; cancel is pressed for 0.5 s while the device switches on: ignored (D-31), one shock.",
        )
        .with(|s| {
            s.script.push((Trigger::At(50), Action::PressCancel(500)));
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
            s.expect = Expect::NeverCountsDown;
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
            "Adult VF; main voltage sensor reads +1% +7 V, supervisor's -1% -7 V (opposite errors at the edge of D-28).",
        )
        .with(|s| {
            s.plant.main_voltage_sensor.gain = 1.01;
            s.plant.main_voltage_sensor.offset = 7.0;
            s.plant.supervisor_voltage_sensor.gain = 0.99;
            s.plant.supervisor_voltage_sensor.offset = -7.0;
        }),
        Scenario::base(
            "f12-compressions-150-over-flatline",
            "Flatline; the responder never stops compressing, at 150/min (finding F-12): never shocked.",
        )
        .with(|s| {
            s.rhythm = Rhythm::Asystole;
            s.after_shock = Vec::new();
            s.responder.keeps_compressing = true;
            s.responder.compression_per_min = 150.0;
            s.expect = Expect::NeverCountsDown;
        }),
        Scenario::base(
            "f12-compressions-150-over-normal-rhythm",
            "Normal rhythm; the responder never stops compressing, at 150/min: never shocked.",
        )
        .with(|s| {
            s.rhythm = Rhythm::sinus(75.0);
            s.after_shock = Vec::new();
            s.responder.keeps_compressing = true;
            s.responder.compression_per_min = 150.0;
            s.expect = Expect::NeverCountsDown;
        }),
        Scenario::base(
            "stops-very-late",
            "Adult VF; the responder takes 3.5 s to stop compressions when told: the analysis waits for hands off and the shock still comes.",
        )
        .with(|s| {
            s.responder.reaction_ms = 3_500;
        }),
        Scenario::base(
            "touch-during-countdown",
            "Adult VF; someone presses on the chest for 0.8 s, 1.5 s into the countdown: shock held back, analysed again, then given.",
        )
        .with(|s| {
            s.script.push((after(Countdown, 1_500), Action::Touch(800)));
        }),
        Scenario::base(
            "ventilations-during-analysis",
            "Adult VF; breaths every 6 s (no compressions) from the start of the first analysis: the analysis is not blocked.",
        )
        .with(|s| {
            s.script.push((after(AnalysingCharging, 0), Action::Ventilate(true)));
        }),
        Scenario::base(
            "strong-pulse",
            "Normal rhythm 75/min with a strong heartbeat in the pad impedance (0.1 ohm): not read as compressions, never charged for a shock.",
        )
        .with(|s| {
            s.rhythm = Rhythm::sinus(75.0);
            s.after_shock = Vec::new();
            s.cardiac_ohms = 0.1;
            s.expect = Expect::NeverCountsDown;
        }),
        Scenario::base(
            "fast-rhythm-with-pulse",
            "Fast normal rhythm 160/min with a 0.1 ohm heartbeat in the pad impedance: not read as compressions.",
        )
        .with(|s| {
            s.rhythm = Rhythm::sinus(160.0);
            s.after_shock = Vec::new();
            s.cardiac_ohms = 0.1;
            s.expect = Expect::InvariantsOnly;
        }),
        Scenario::base(
            "small-compressions",
            "Adult VF; compressions change the impedance by only 0.2 ohm (seen in some real patients): below the detector's threshold.",
        )
        .with(|s| {
            s.responder.compression_ohms = 0.2;
            s.expect = Expect::InvariantsOnly;
        }),
    ];

    v.push(fault_scenario(
        Fault::ImpedanceChannelLost,
        Trigger::At(0),
        Expect::Shocks(1),
        "from switch-on; the device carries on from the heart signal alone",
    ));
    v.push(fault_scenario(
        Fault::MainVoltageSensorWrong,
        after(AnalysingCharging, 3_000),
        Expect::FaultNoShock,
        "while charging during the analysis",
    ));
    v.push(fault_scenario(
        Fault::SupervisorVoltageSensorWrong,
        after(AnalysingCharging, 3_000),
        Expect::FaultNoShock,
        "while charging during the analysis",
    ));
    v.push(fault_scenario(
        Fault::ChargerWeak,
        after(RescueState::AttachPads, 0),
        Expect::FaultNoShock,
        "from just after the self-test",
    ));
    v.push(fault_scenario(
        Fault::ChargerRunaway,
        after(AnalysingCharging, 2_500),
        Expect::FaultNoShock,
        "while charging during the analysis",
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
        after(AnalysingCharging, 3_000),
        Expect::FaultNoShock,
        "while charging during the analysis",
    ));
    v.push(fault_scenario(
        Fault::PadsOff,
        after(Countdown, 2_000),
        Expect::AbortedNoShock,
        "2 s into the countdown; nobody puts them back",
    ));
    v.push(fault_scenario(
        Fault::CancelPressed,
        after(AnalysingCharging, 2_000),
        Expect::NoShock,
        "2 s into the first analysis",
    ));
    v.push(fault_scenario(
        Fault::CancelStuck,
        Trigger::At(20_000),
        Expect::EndsInFault,
        "from 20 s, during CPR; reported as stuck after 10 s",
    ));
    v.push(fault_scenario(
        Fault::BatteryDepleted,
        after(AnalysingCharging, 3_000),
        Expect::FaultNoShock,
        "while charging during the analysis",
    ));
    v.push(fault_scenario(
        Fault::BridgeFailsToFire,
        Trigger::At(0),
        Expect::NoShock,
        "from switch-on",
    ));
    v.extend(heat_scenarios());
    v
}

/// Heat scenarios (D-41, D-48, D-51). Temperatures from the one-node
/// thermal model in [`crate::thermal`] (parts study figures; simulated).
fn heat_scenarios() -> Vec<Scenario> {
    use crate::thermal::TempSensorFault;
    use aed_core::TempPoint;
    vec![
        Scenario::base(
            "hot-device-rescue",
            "Adult VF; the device is switched on at 63 °C (out of a hot car) into 25 °C air: \
             CPR guidance at once, analyses without charging, \"Shock advised. Device too hot \
             to charge.\", then shocks once it has cooled below 59 °C (D-51)",
        )
        .with(|s| {
            s.interior_c = 63.0;
            s.ambient_c = 25.0;
            s.duration_ms = 420_000;
        }),
        Scenario::base(
            "warm-device-rescue",
            "Adult VF at 57 °C inside, 50 °C air: above the 55 °C pre-charge limit the \
             device charges only once a shock is advised, then shocks",
        )
        .with(|s| {
            s.interior_c = 57.0;
            s.ambient_c = 50.0;
        }),
        Scenario::base(
            "temp-sensor-failed-main",
            "Adult VF; the main controller's capacitor temperature sensor is dead from \
             switch-on: the hottest other reading plus 5 °C stands in, and it shocks",
        )
        .with(|s| {
            s.script.push((
                Trigger::At(0),
                Action::TempSensor(TempPoint::Capacitor, TempSensorFault::Missing),
            ));
        }),
        Scenario::base(
            "temp-sensor-stuck-supervisor",
            "Adult VF; the supervisor's temperature sensor is stuck at 80 °C: it is \
             distrusted (by both controllers) and the device shocks",
        )
        .with(|s| {
            s.script.push((
                Trigger::At(0),
                Action::TempSensor(TempPoint::Supervisor, TempSensorFault::StuckAt(80.0)),
            ));
        }),
    ]
}

/// The named scenario, if it exists.
#[must_use]
pub fn by_name(name: &str) -> Option<Scenario> {
    catalogue().into_iter().find(|s| s.name == name)
}
