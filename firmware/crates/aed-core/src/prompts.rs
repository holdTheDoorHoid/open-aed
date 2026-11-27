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
    /// A shock is advised and the capacitor is already charged (it charged
    /// during the analysis, D-29): the countdown follows at once.
    ShockAdvised,
    /// A shock is advised and the capacitor is still charging.
    ShockAdvisedCharging,
    /// Charged; the automatic countdown is starting ("Stand clear. Three,
    /// two, one.", D-30).
    StandClear,
    /// One number of the countdown (3, 2, 1 with the default 3 s countdown).
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
    /// The pads are on but read above the shock range (poor contact, D-32).
    /// Said as an analysis starts and repeated during CPR; never while the
    /// responder must keep hands off.
    PressPadsFirmly,
    /// A shock was due but the pads still read above the shock range: no
    /// shock (D-32). Always followed at once by "Start CPR."
    PadContactPoorNoShock,
    /// The cancel control has read pressed for too long and counts as stuck
    /// (D-31). A stuck cancel would block every shock, so this is a fault.
    CancelStuckContinueCpr,
    /// Compression sensing (D-36) saw the chest being compressed or moved
    /// while a shock was being prepared: the shock is held back and the
    /// rhythm is analysed again. Always followed at once by "Analysing. Do
    /// not touch the patient."
    MovementShockDelayed,
    /// As [`Prompt::MovementShockDelayed`], but the device has already
    /// analysed again once in this cycle: no shock now. Always followed at
    /// once by "Start CPR."
    MovementNoShock,
    /// During CPR, no compressions have been sensed for a while (D-14).
    ContinueCpr,
    /// Said once at the end of the switch-on self-test when the device is
    /// too hot to charge its capacitor (D-51). Informational: the next
    /// instruction ("Attach pads" or "Analysing") follows at once.
    DeviceTooHot,
    /// A shock is advised but the device is too hot to charge (D-51). Always
    /// followed at once by "Start CPR."; the device analyses again after the
    /// CPR period and shocks once it has cooled.
    ShockAdvisedTooHot,
    /// A shock being prepared was stopped because the supervisor's own
    /// temperature reading went over its limit. Always followed at once by
    /// "Start CPR."
    TooHotNoShock,
    /// The pads have used up their heat allowance (D-52). Advisory only,
    /// like out-of-date pads.
    ReplacePadsHeat,
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
            Prompt::ShockAdvised => "Shock advised.",
            Prompt::ShockAdvisedCharging => "Shock advised. Charging. Stand clear.",
            Prompt::StandClear => "Stand clear.",
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
            Prompt::PressPadsFirmly => "Press pads firmly to bare chest.",
            Prompt::PadContactPoorNoShock => "Poor pad contact. No shock.",
            Prompt::CancelStuckContinueCpr => "Cancel button stuck. No shocks. Continue CPR.",
            Prompt::MovementShockDelayed => "Movement detected. Shock delayed.",
            Prompt::MovementNoShock => "Movement detected. No shock.",
            Prompt::ContinueCpr => "Continue CPR.",
            Prompt::DeviceTooHot => "Device too hot. Shocks delayed until it cools.",
            Prompt::ShockAdvisedTooHot => "Shock advised. Device too hot to charge.",
            Prompt::TooHotNoShock => "Device too hot. No shock.",
            Prompt::ReplacePadsHeat => "Pads heat damaged. Replace pads after use.",
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
