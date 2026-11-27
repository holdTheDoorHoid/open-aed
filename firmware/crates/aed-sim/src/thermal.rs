// SPDX-License-Identifier: GPL-3.0-or-later
//! A one-node thermal model of the device, and its six temperature sensors.
//!
//! SIMULATION ONLY. The inside of the box is one temperature that follows
//! the surroundings with a first-order lag and is warmed by the charges
//! burned inside it. Every figure is from the parts study (docs/PARTS_STUDY.md
//! §4, marked [CALC] there), not measured:
//!
//! * heat capacity 630 J/K (0.7 kg × 900 J/kg·K);
//! * conductance to still air 0.49 W/K (494 cm² × about 10 W/m²·K), so
//!   the time constant is about 21 minutes;
//! * every joule burned in the internal discharge load stays inside;
//! * the charger loses about 63 J inside for every 190 J it puts into the
//!   capacitor (0.33 J per joule);
//! * the electronics and audio dissipate about 0.5 W while switched on
//!   [EST in the study].
//!
//! Not modelled: separate temperatures for the capacitor, the cells and the
//! discharge resistors (the study expects the area near the resistors to run
//! hotter: a per-sensor offset stands in for that), sunlight on the case,
//! the insulated pouch, and how the cells heat when they deliver current.

use aed_core::{DeciC, SensorSet, TempPoint};

/// The model's constants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalConfig {
    /// J/K. Parts study §4: 630.
    pub heat_capacity_j_per_k: f64,
    /// W/K to the surroundings. Parts study §4: 0.49.
    pub conductance_w_per_k: f64,
    /// Electronics and audio while switched on, W. Parts study §4: 0.5 [EST].
    pub electronics_w: f64,
    /// Heat left inside by the charger per joule it puts into the
    /// capacitor. Parts study §4: 63 / 190 = 0.33.
    pub charger_loss_per_j: f64,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        ThermalConfig {
            heat_capacity_j_per_k: 630.0,
            conductance_w_per_k: 0.49,
            electronics_w: 0.5,
            charger_loss_per_j: 63.0 / 190.0,
        }
    }
}

impl ThermalConfig {
    /// The time constant, seconds.
    #[must_use]
    pub fn time_constant_s(&self) -> f64 {
        self.heat_capacity_j_per_k / self.conductance_w_per_k
    }
}

/// What is wrong with one temperature sensor.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TempSensorFault {
    /// Reads the truth (plus its offset).
    #[default]
    Healthy,
    /// Does not answer.
    Missing,
    /// Reads this, whatever the truth.
    StuckAt(f64),
}

/// The device's inside temperature and its sensors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thermal {
    /// The constants.
    pub config: ThermalConfig,
    /// The surroundings, °C.
    pub ambient_c: f64,
    /// The inside of the device, °C.
    pub interior_c: f64,
    /// Each sensor's offset from the inside temperature, °C, in
    /// [`TempPoint::ALL`] order.
    pub offsets: [f64; 6],
    /// Each sensor's fault, in [`TempPoint::ALL`] order.
    pub faults: [TempSensorFault; 6],
    /// Highest inside temperature so far, °C.
    pub peak_c: f64,
}

impl Thermal {
    /// Everything at `c` °C.
    #[must_use]
    pub fn at(c: f64) -> Self {
        Thermal {
            config: ThermalConfig::default(),
            ambient_c: c,
            interior_c: c,
            offsets: [0.0; 6],
            faults: [TempSensorFault::Healthy; 6],
            peak_c: c,
        }
    }

    /// Advance `dt_s` seconds with `heat_j` joules released inside and the
    /// electronics on or off. Exact for constant surroundings and power.
    pub fn step(&mut self, dt_s: f64, heat_j: f64, electronics_on: bool) {
        let cfg = self.config;
        let power = heat_j / dt_s.max(1e-9) + if electronics_on { cfg.electronics_w } else { 0.0 };
        let target = self.ambient_c + power / cfg.conductance_w_per_k;
        let k = (-dt_s / cfg.time_constant_s()).exp();
        self.interior_c = target + (self.interior_c - target) * k;
        self.peak_c = self.peak_c.max(self.interior_c);
    }

    /// The true temperature at a sensor point (inside temperature plus the
    /// point's offset).
    #[must_use]
    pub fn true_at(&self, p: TempPoint) -> f64 {
        self.interior_c + self.offsets.get(p.index()).copied().unwrap_or(0.0)
    }

    /// What the sensor at `p` reads, in tenths of a degree.
    #[must_use]
    pub fn reading(&self, p: TempPoint) -> Option<DeciC> {
        let to_dc = |c: f64| DeciC((c * 10.0).round().clamp(-3_000.0, 3_000.0) as i16);
        match self.faults.get(p.index()).copied().unwrap_or_default() {
            TempSensorFault::Healthy => Some(to_dc(self.true_at(p))),
            TempSensorFault::Missing => None,
            TempSensorFault::StuckAt(c) => Some(to_dc(c)),
        }
    }

    /// Every sensor's reading.
    #[must_use]
    pub fn sensor_set(&self) -> SensorSet {
        let mut s = SensorSet::default();
        for p in TempPoint::ALL {
            s.set(p, self.reading(p));
        }
        s
    }

    /// The hottest true temperature at the capacitor or a cell, °C.
    #[must_use]
    pub fn hottest_charge_part_c(&self) -> f64 {
        [TempPoint::Capacitor, TempPoint::Cell1, TempPoint::Cell2]
            .iter()
            .map(|p| self.true_at(*p))
            .fold(f64::MIN, f64::max)
    }
}
