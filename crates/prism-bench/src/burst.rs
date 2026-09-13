#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Baseline,
    Burst,
    Recovery,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Burst => "burst",
            Self::Recovery => "recovery",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Calibration {
    pub throughputs: Vec<f64>,
    pub median_frames_per_sec: f64,
    pub baseline_frames_per_sec: f64,
    pub burst_frames_per_sec: f64,
    pub baseline_interval_ns: u128,
    pub burst_interval_ns: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurstError {
    InvalidRates,
    InvalidFrameCount,
    IntervalOverflow,
}

fn interval_for(rate: f64) -> Result<u128, BurstError> {
    if !rate.is_finite() || rate <= 0.0 {
        return Err(BurstError::InvalidRates);
    }
    let interval = (1_000_000_000.0 / rate).ceil();
    if !interval.is_finite() || interval < 1.0 || interval > u128::MAX as f64 {
        return Err(BurstError::IntervalOverflow);
    }
    Ok(interval as u128)
}

impl Calibration {
    pub fn from_throughputs(throughputs: &[f64]) -> Result<Self, BurstError> {
        if throughputs.len() != 5 || throughputs.iter().any(|rate| !rate.is_finite() || *rate <= 0.0) {
            return Err(BurstError::InvalidRates);
        }
        let mut ordered = throughputs.to_vec();
        ordered.sort_by(f64::total_cmp);
        let median = ordered[2];
        let baseline = median * 0.8;
        let burst = baseline * 10.0;
        Ok(Self {
            throughputs: throughputs.to_vec(),
            median_frames_per_sec: median,
            baseline_frames_per_sec: baseline,
            burst_frames_per_sec: burst,
            baseline_interval_ns: interval_for(baseline)?,
            burst_interval_ns: interval_for(burst)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseSchedule {
    phase: Phase,
    interval_ns: u128,
    offered_frames_per_sec: f64,
    arrival_offsets_ns: Vec<u128>,
}

impl PhaseSchedule {
    pub fn new(calibration: &Calibration, phase: Phase, frames: usize) -> Result<Self, BurstError> {
        if frames == 0 {
            return Err(BurstError::InvalidFrameCount);
        }
        let (interval_ns, rate) = match phase {
            Phase::Baseline | Phase::Recovery => (
                calibration.baseline_interval_ns,
                calibration.baseline_frames_per_sec,
            ),
            Phase::Burst => (calibration.burst_interval_ns, calibration.burst_frames_per_sec),
        };
        let arrival_offsets_ns = (0..frames)
            .map(|index| interval_ns.checked_mul(index as u128).ok_or(BurstError::IntervalOverflow))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { phase, interval_ns, offered_frames_per_sec: rate, arrival_offsets_ns })
    }

    pub fn phase(&self) -> Phase { self.phase }
    pub fn frames(&self) -> usize { self.arrival_offsets_ns.len() }
    pub fn interval_ns(&self) -> u128 { self.interval_ns }
    pub fn offered_frames_per_sec(&self) -> f64 { self.offered_frames_per_sec }
    pub fn arrival_offset_ns(&self, index: usize) -> u128 { self.arrival_offsets_ns[index] }
}
