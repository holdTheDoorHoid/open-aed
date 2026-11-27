// SPDX-License-Identifier: GPL-3.0-or-later
//! What each controller can see and do, described as two traits.
//!
//! `aed-core` never touches a pin. It asks the hardware for readings and
//! gives it commands through these two traits. The real implementations will
//! live in a board-support layer once a processor is chosen; for now the only
//! implementations are the test mocks in this crate and the simulator.
//!
//! The two traits are deliberately separate. The supervisor has its own
//! clock, its own voltage sensor, its own pad-resistance reading and its own
//! view of the cancel button. It can see nothing the main controller sees
//! except the status message the main controller sends it.
//!
//! **Discharge by default.** The storage capacitor is connected to the
//! internal discharge load unless BOTH controllers are actively holding the
//! discharge path open. Either one letting go empties the capacitor, and the
//! charger cannot run while the discharge path is active (a hardware
//! requirement). See [`crate::interlock`].

use crate::link::{MainStatus, SupervisorReport};
use crate::prompts::Prompt;
use aed_types::{Millis, Ohms, PatientMode, ShockPlan, Volts};

/// State of the battery as the power monitor reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    /// Enough charge for a full rescue.
    Good,
    /// Low: the device keeps working but says so.
    Low,
    /// Too low to charge the capacitor: a fault.
    Depleted,
}

/// State of the pad set plugged into the device (pads stay plugged in).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadCartridge {
    /// Pads present and within date.
    Ok,
    /// No pads plugged in.
    Missing,
    /// Pads plugged in but past their use-by date.
    Expired,
}

/// What the shock hardware reports after a shock was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockStatus {
    /// No shock is requested.
    Idle,
    /// A shock is requested and has not happened yet (for example, the
    /// supervisor's permit has not arrived).
    Pending,
    /// The shock was delivered.
    Delivered,
    /// The hardware interlock refused the shock.
    Refused,
    /// The output stage reported a fault while delivering.
    Fault,
}

/// Everything the MAIN controller can see and do.
pub trait MainHardware {
    /// The main controller's clock.
    fn now(&self) -> Millis;
    /// Resistance between the pads right now. Pads that are off read very high.
    fn pad_resistance(&mut self) -> Ohms;
    /// Capacitor voltage on the main controller's sensor.
    fn capacitor_voltage(&mut self) -> Volts;
    /// Turn the charger on, charging (and holding) to `Some(target)`, or off with `None`.
    fn set_charger(&mut self, target: Option<Volts>);
    /// Hold the discharge path open (`true`, capacitor may stay charged) or
    /// release it (`false`, capacitor empties into the internal load).
    fn set_discharge_hold(&mut self, hold: bool);
    /// Raise the shock request line, with the plan's phase timings loaded into
    /// the output stage. Called once per shock.
    fn request_shock(&mut self, plan: &ShockPlan);
    /// Lower the shock request line. Safe to call at any time, repeatedly.
    fn withdraw_shock_request(&mut self);
    /// What the shock hardware reports about the current request.
    fn shock_status(&mut self) -> ShockStatus;
    /// True while the cancel control is pressed.
    fn cancel_pressed(&mut self) -> bool;
    /// The position of the adult/child switch.
    fn patient_mode(&mut self) -> PatientMode;
    /// The battery as the power monitor sees it now.
    fn battery(&mut self) -> BatteryStatus;
    /// Put the battery under a realistic load briefly and report the result
    /// (used by the self-test).
    fn battery_under_load(&mut self) -> BatteryStatus;
    /// State of the plugged-in pad set (presence and use-by date).
    fn pad_cartridge(&mut self) -> PadCartridge;
    /// Speak and show a prompt.
    fn prompt(&mut self, prompt: Prompt);
    /// One beat of the CPR compression metronome (a click or tone).
    fn metronome_beat(&mut self);
    /// Set the ready / not-ready indicator.
    fn set_ready_indicator(&mut self, ready: bool);
    /// Send a status (heartbeat) message to the supervisor.
    fn send_status(&mut self, status: &MainStatus);
    /// The latest report from the supervisor not yet read, if any.
    fn supervisor_report(&mut self) -> Option<SupervisorReport>;
}

/// Everything the SUPERVISOR can see and do. It is a different chip with its
/// own sensors; nothing here is shared with [`MainHardware`].
pub trait SupervisorHardware {
    /// The supervisor's own clock.
    fn now(&self) -> Millis;
    /// Capacitor voltage on the supervisor's own, independent sensor.
    fn capacitor_voltage(&mut self) -> Volts;
    /// Pad resistance on the supervisor's own measurement.
    fn pad_resistance(&mut self) -> Ohms;
    /// The supervisor's own view of the cancel control.
    fn cancel_pressed(&mut self) -> bool;
    /// Raise (`true`) or lower (`false`) the shock permit line.
    fn set_permit(&mut self, permit: bool);
    /// Hold the discharge path open (`true`) or release it (`false`).
    fn set_discharge_hold(&mut self, hold: bool);
    /// The latest status message from the main controller not yet read, if any.
    fn receive_status(&mut self) -> Option<MainStatus>;
    /// Send a report back to the main controller.
    fn send_report(&mut self, report: &SupervisorReport);
    /// Switch the supervisor's own fault alarm on (`true`) or off (`false`).
    ///
    /// A dedicated buzzer and light, wired to the supervisor alone and
    /// labelled on the case (for example "Device fault. No shocks. Continue
    /// CPR."), so that a device whose main controller has died is never
    /// silent. The supervisor switches it on whenever it has a latched fault,
    /// and if the main controller has never sent a message since switch-on.
    fn set_fault_alarm(&mut self, on: bool);
}
