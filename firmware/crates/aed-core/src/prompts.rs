// SPDX-License-Identifier: GPL-3.0-or-later
//! Every message the device speaks or shows.
//!
//! The users are trained responders (police, paramedics), so the wording is
//! short. All wording is provisional and has not been reviewed by clinicians
//! or tested with responders.

/// One spoken and displayed message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    /// Pads are not on a patient.
    AttachPads,
    /// The device is set for a child: says so and asks for front-and-back pad placement.
    ChildModeFrontBack,
    /// The device is set for an adult.
    AdultMode,
    /// The pad contact is poor or was lost.
    CheckPads,
    /// The pads are out of date or missing (found by the self-test). Advisory only.
    ReplacePads,
    /// Analysis has started; nobody may touch the patient.
    AnalysingDoNotTouch,
    /// The CPR period is over; stop compressions for analysis.
    StopCprAnalysing,
    /// The analysis is "not sure" (usually movement): repeats the hands-off
    /// instruction while the device keeps analysing.
    DoNotTouchPatient,
    /// The analysis could not reach an answer in time. No shock. Always
    /// followed at once by "Start CPR."
    AnalysisInconclusive,
    /// A shock is advised and the capacitor is charging.
    ShockAdvisedCharging,
    /// Charged; the automatic countdown is starting.
    StandClear,
    /// One number of the countdown (5, 4, 3, 2, 1).
    Countdown(u8),
    /// The shock was delivered.
    ShockDelivered,
    /// A shock was advised but was not given (refused by the second check,
    /// the confirmation went stale, or the mode was changed).
    ShockNotDelivered,
    /// The analysis found a rhythm that must not be shocked.
    NoShockAdvised,
    /// Start or continue chest compressions.
    StartCpr,
    /// The cancel control stopped a shock that was being prepared.
    ShockCancelled,
    /// The cancel control stopped an analysis.
    AnalysisCancelled,
    /// The rhythm stopped being shockable before delivery; no shock.
    RhythmChangedNoShock,
    /// The device has a fault and will not shock; keep doing CPR.
    DeviceFaultContinueCpr,
    /// The device could not confirm its capacitor is empty. The most serious fault.
    SeriousFaultContinueCpr,
    /// The battery is low.
    LowBattery,
}

impl Prompt {
    /// The plain-English words for this prompt.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Prompt::AttachPads => "Attach pads to bare chest.",
            Prompt::ChildModeFrontBack => "Child mode. One pad on the chest, one on the back.",
            Prompt::AdultMode => "Adult mode.",
            Prompt::CheckPads => "Check pads.",
            Prompt::ReplacePads => "Pads out of date. Replace pads after use.",
            Prompt::AnalysingDoNotTouch => "Analysing. Do not touch the patient.",
            Prompt::StopCprAnalysing => "Stop CPR. Analysing. Do not touch the patient.",
            Prompt::DoNotTouchPatient => "Do not touch the patient. Still analysing.",
            Prompt::AnalysisInconclusive => "Unable to analyse. No shock.",
            Prompt::ShockAdvisedCharging => "Shock advised. Charging. Stand clear.",
            Prompt::StandClear => "Stand clear. Shocking in",
            Prompt::Countdown(n) => countdown_word(n),
            Prompt::ShockDelivered => "Shock delivered.",
            Prompt::ShockNotDelivered => "Shock not delivered.",
            Prompt::NoShockAdvised => "No shock advised.",
            Prompt::StartCpr => "Start CPR.",
            Prompt::ShockCancelled => "Shock cancelled.",
            Prompt::AnalysisCancelled => "Analysis cancelled.",
            Prompt::RhythmChangedNoShock => "Rhythm changed. No shock.",
            Prompt::DeviceFaultContinueCpr => "Device fault. No shocks. Continue CPR.",
            Prompt::SeriousFaultContinueCpr => {
                "Serious device fault. Do not touch the device. Continue CPR."
            }
            Prompt::LowBattery => "Battery low.",
        }
    }
}

/// The spoken word for a countdown number.
const fn countdown_word(n: u8) -> &'static str {
    match n {
        1 => "One.",
        2 => "Two.",
        3 => "Three.",
        4 => "Four.",
        5 => "Five.",
        6 => "Six.",
        7 => "Seven.",
        8 => "Eight.",
        9 => "Nine.",
        10 => "Ten.",
        _ => "",
    }
}
