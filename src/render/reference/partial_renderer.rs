use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::btree_map;


use signal::Signal;
use phaser::PhaserCoeff;
use real::Real32;

use render::render_spec::RenderSpec;

/// Any angular frequencies within this distance from eachother will be
/// considered equal, and the difference is attributed to float rounding.
const FREQ_DELTA : f32 = 0.00001f32;
/// Any waves with amplitude less than this delta are considered safe to drop
const AMP_DELTA : f32 = 0.000000001f32;
/// Provides a type that compares equal if to frequencies are nearly
/// indistinguishable.
/// The threshold for "indistinguishable" is not whether or not they are
/// audibly different, but rather, could both frequencies feasibly be obtained
/// from the same calculation by reordering the mathematical operations?
#[derive(Debug)]
struct ApproxFreq (Real32);

/// Takes a series of Partials and turns them into a PCM/audio signal.
//#[derive(Debug)]
pub struct PartialRenderer {
    // Note: at 44100 Hz, u32 can cover 1.12 days of audio
    // u64 = 12,000,000 years
    frame_idx : u64,
    inv_sample_rate : f64,
    /// Maps the angular frequency of a wave to its amplitude coefficient.
    partials : BTreeMap<ApproxFreq, PhaserCoeff>,
}

impl PartialRenderer {
    /// Creates a new renderer according to the provided `spec`
    pub fn new(spec: RenderSpec) -> PartialRenderer {
        PartialRenderer {
            partials: BTreeMap::new(),
            frame_idx: 0,
            inv_sample_rate: 1.0f64/(spec.sample_rate() as f64)
        }
    }
    pub fn feed(&mut self, signal : Signal) {
        // If there's already an entry for a frequency very close to ours,
        // then add our coefficient into that entry. Otherwise, create a new
        // entry. In either case, delete the entry if the amplitude of the wave
        // is less than some threshold.
        //
        // Deleting the entry means that calling feed(p) followed by feed(-p)
        // will always work to remove p from the table.
        match self.partials.entry(ApproxFreq(signal.ang_freq())) {
            btree_map::Entry::Vacant(entry) => {
                let new_val = signal.phaser_coeff();
                if new_val.norm_sqr() >= Real32::new(AMP_DELTA*AMP_DELTA) {
                    entry.insert(new_val);
                }
            },
            btree_map::Entry::Occupied(entry_) => {
                let mut entry = entry_;
                let new_val = entry.get() + signal.phaser_coeff();
                if new_val.norm_sqr() >= Real32::new(AMP_DELTA*AMP_DELTA) {
                    entry.insert(new_val);
                } else {
                    entry.remove();
                }
            }
        };
    }
    /// Obtain the next sample
    pub fn step(&mut self) -> f32 {
        let seconds = self.frame_idx as f64 * self.inv_sample_rate;
        self.frame_idx += 1;

        // Signal is described by sum: coeff*exp(i*freq*seconds)
        // we only care about the real portion of the signal
        // exp(i*w) = cos(w) + i*sin(w)
        // Therefore signal = sum: coeff*Complex32(cos(w), sin(w)).re
        self.partials.iter().fold(Real32::new(0.0f32), |accum, (freq, coeff)| {
            let (res_sin, res_cos) = f64::sin_cos(seconds*freq.0.value() as f64);
            accum + (coeff*PhaserCoeff::new_f32(res_cos as f32, res_sin as f32)).re()
        }).value()
    }
}



impl PartialEq for ApproxFreq {
    /// Returns true if the frequencies are within +/- FREQ_DELTA from eeachother
    /// In order to make NAN's considered equal, we check for the inverse and negate it.
    fn eq(&self, other: &ApproxFreq) -> bool {
        !(self.0 > other.0 + Real32::new(FREQ_DELTA)
            || other.0 > self.0 + Real32::new(FREQ_DELTA)
        )
    }
}

impl PartialOrd for ApproxFreq {
    fn partial_cmp(&self, other: &ApproxFreq) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ApproxFreq {
    fn cmp(&self, other: &ApproxFreq) -> Ordering {
        if self.eq(other) {
            Ordering::Equal
        } else if self.0 < other.0 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl Eq for ApproxFreq {}
