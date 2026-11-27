// SPDX-License-Identifier: GPL-3.0-or-later
//! Scores the compression detector (`aed-cpr`) on real recordings.
//!
//! DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE. See SAFETY.md.
//!
//! The recordings are the Paris Fire Brigade's transthoracic impedance
//! dataset (Mendeley Data, DOI 10.17632/34gffrm87w.1, CC BY 4.0). They are
//! not part of this repository; see `REPORT.md` for where to get them and
//! where to put them.
//!
//! * [`data`]: reading the files (treated as untrusted input).
//! * [`reference`]: where the compressions were, worked out after the fact.
//! * [`score`]: running the detector as the device would and comparing.
//! * [`split`]: the fixed development / held-out case lists and their hashes.
//! * [`summary`]: totals over many cases, as text.
// A development-computer tool: an out-of-range index here is a bug in the
// tool, found by its tests, not a device hazard.
#![allow(clippy::indexing_slicing)]

pub mod data;
pub mod reference;
pub mod score;
pub mod split;
pub mod summary;
