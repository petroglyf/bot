//! glTF character loader: extracts animation keyframes and the kinematic tree
//! from a `.glb` file.
//!
//! Replaces `Loader.cpp` + `DataPool.cpp` from the original C++ implementation.
//! Uses the [`gltf`] crate; no C++ Assimp dependency.

use std::collections::HashMap;
use std::path::Path;

use gltf::animation::util::ReadOutputs;

use crate::types::{AnimJoint, Animation, Joint, JointInitParams, KineTree};

/// Strips Maya FBX export suffixes to produce a clean joint name.
///
/// Handles patterns like `LArm_Joint1` → `LArm` and `LArm_IK` → `LArm`.
fn canonical_name(s: &str) -> &str {
    if let Some(pos) = s.find("_Joint") {
        return &s[..pos];
    }
    if let Some(pos) = s.find('_') {
        return &s[..pos];
    }
    s
}

/// Loads a character from a `.glb` file.
///
/// Returns the first animation in the file and the full kinematic tree built
/// from the glTF node hierarchy.
///
/// # Errors
///
/// Returns an error string if the file cannot be opened or contains no
/// animation data.
pub fn load_character(path: &Path) -> Result<(Animation, KineTree), String> {
    let (doc, buffers, _images) =
        gltf::import(path).map_err(|e| format!("gltf import failed: {e}"))?;

    let anim = doc
        .animations()
        .next()
        .ok_or("nao.glb contains no animations")?;

    let anim_name = anim.name().unwrap_or("unnamed").to_string();

    let mut joints: Vec<AnimJoint> = Vec::new();
    let mut duration: f64 = 0.0;

    for channel in anim.channels() {
        let target_node = channel.target().node();
        let name = canonical_name(target_node.name().unwrap_or("")).to_string();

        let reader = channel.reader(|buf| Some(&buffers[buf.index()]));

        // Only care about rotation channels (analogous to Loader.cpp's check)
        let Some(ReadOutputs::Rotations(rots)) = reader.read_outputs() else {
            continue;
        };

        let times: Vec<f32> = reader
            .read_inputs()
            .ok_or("missing input accessor")?
            .collect();

        let rots_f32: Vec<[f32; 4]> = rots.into_f32().collect();

        // Skip joints with no meaningful rotation (all keyframes identical)
        let has_motion = rots_f32.windows(2).any(|w| {
            w[0].iter()
                .zip(w[1].iter())
                .any(|(a, b)| (a - b).abs() > 1e-5)
        });
        if !has_motion && rots_f32.len() > 1 {
            continue;
        }

        let keyframes: Vec<Joint> = times
            .iter()
            .zip(rots_f32.iter())
            .map(|(&t, rot)| {
                // Axis-angle magnitude from quaternion: angle = 2*acos(w)
                let w = rot[3].clamp(-1.0, 1.0);
                let angle = 2.0 * w.acos();
                Joint { rpos: angle, t }
            })
            .collect();

        if let Some(&last_t) = times.last() {
            if f64::from(last_t) > duration {
                duration = f64::from(last_t);
            }
        }

        joints.push(AnimJoint { name, keyframes });
    }

    let animation = Animation {
        name: anim_name,
        duration,
        joints,
    };

    let kine_tree = build_kine_tree(&doc);

    Ok((animation, kine_tree))
}

/// Walks the glTF node hierarchy to build the kinematic tree.
fn build_kine_tree(doc: &gltf::Document) -> KineTree {
    let mut details: HashMap<String, JointInitParams> = HashMap::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();

    for node in doc.nodes() {
        let parent_name = canonical_name(node.name().unwrap_or("")).to_string();

        let (translation, rotation, _scale) = node.transform().decomposed();

        details.insert(
            parent_name.clone(),
            JointInitParams {
                rel_pos: translation,
                init_rot: rotation,
            },
        );

        let child_names: Vec<String> = node
            .children()
            .map(|c| canonical_name(c.name().unwrap_or("")).to_string())
            .collect();

        children.insert(parent_name, child_names);
    }

    KineTree { details, children }
}
