//! Core data types for the SMURFS animation and kinematics system.
//!
//! These mirror the C++ `DataTypes.hpp` structs but use idiomatic Rust.

use std::collections::HashMap;

/// A single joint keyframe: rotation in radians at a given time.
#[derive(Debug, Clone, Copy)]
pub struct Joint {
    /// Rotation in radians.
    pub rpos: f32,
    /// Time in seconds.
    pub t: f32,
}

/// All keyframes for one named joint.
#[derive(Debug, Clone)]
pub struct AnimJoint {
    /// Canonical joint name (Maya suffix stripped).
    pub name: String,
    /// Time-ordered keyframes.
    pub keyframes: Vec<Joint>,
}

/// A loaded animation: name, total duration, and per-joint trajectories.
#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    /// Duration in seconds.
    pub duration: f64,
    pub joints: Vec<AnimJoint>,
}

/// Initial pose of a joint relative to its parent.
#[derive(Debug, Clone, Copy)]
pub struct JointInitParams {
    /// Translation from parent in metres: [x, y, z].
    pub rel_pos: [f32; 3],
    /// Initial rotation quaternion [x, y, z, w].
    pub init_rot: [f32; 4],
}

/// Kinematic tree: parent→children relationships and per-joint initial poses.
#[derive(Debug, Clone, Default)]
pub struct KineTree {
    /// Map from joint name to its initial pose parameters.
    pub details: HashMap<String, JointInitParams>,
    /// Map from joint name to its children's names.
    pub children: HashMap<String, Vec<String>>,
}
