//! Animation graph: maintains the current blended joint pose and advances it
//! each tick.
//!
//! Currently supports a single looping static animation, matching the
//! behaviour wired up in the original `DesktopEntry.java`.

use crate::data_pool;

/// Current blended pose: one `f64` radians value per joint, in joint-index
/// order.
#[derive(Debug, Default)]
pub struct AnimGraph {
    /// Joint positions in radians, indexed by joint order from the loader.
    pub pose: Vec<f64>,
}

impl AnimGraph {
    /// Advance the animation to time `t` (seconds since start), linearly
    /// interpolating between keyframes.
    ///
    /// Loops the animation when `t` exceeds the animation duration.
    pub fn update(&mut self, t: f64) {
        data_pool::with_state(|ch| {
            let anim = &ch.animation;
            let looped_t = if anim.duration > 0.0 {
                t % anim.duration
            } else {
                0.0
            };

            self.pose.resize(anim.joints.len(), 0.0);

            for (i, joint) in anim.joints.iter().enumerate() {
                // Keyframe times are f32, so narrowing here is intentional.
                #[allow(clippy::cast_possible_truncation)]
                let t32 = looped_t as f32;
                self.pose[i] = interpolate(&joint.keyframes, t32).into();
            }
        });
    }
}

/// Linear interpolation between two adjacent keyframes.
fn interpolate(keyframes: &[crate::types::Joint], t: f32) -> f32 {
    if keyframes.is_empty() {
        return 0.0;
    }
    if keyframes.len() == 1 || t <= keyframes[0].t {
        return keyframes[0].rpos;
    }
    let last = keyframes.last().unwrap();
    if t >= last.t {
        return last.rpos;
    }

    // Find the surrounding pair
    let idx = keyframes.partition_point(|k| k.t <= t).saturating_sub(1);
    let a = &keyframes[idx];
    let b = &keyframes[idx + 1];
    let alpha = (t - a.t) / (b.t - a.t);
    a.rpos + alpha * (b.rpos - a.rpos)
}
