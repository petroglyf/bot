//! SMURFS robotics animation server.
//!
//! Loads a glTF robot model (defaults to `nao.glb`), then serves joint
//! animation data over `ZeroMQ`:
//!
//! - Port **31337** — `FlatBuffers` RPC (`ZMQ` REP)
//! - Port **31338** — 60 Hz joint stream (`ZMQ` PUB)
//!
//! # Usage
//!
//! ```text
//! cargo run -- [path/to/robot.glb]
//! ```

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// The kinematic tree and some metadata are loaded but not yet consumed;
// they are scaffolding for the IK port.
#![allow(dead_code)]

mod anim;
mod behavior;
mod data_pool;
mod loader;
mod server;
mod state;
mod types;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn main() {
    let glb_path: PathBuf = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "nao.glb".to_string())
        .into();

    println!("Loading {}", glb_path.display());
    if let Err(e) = data_pool::load(&glb_path) {
        eprintln!("Fatal: {e}");
        std::process::exit(1);
    }
    data_pool::with_state(|ch| {
        println!(
            "Loaded '{}' — {} joints, {:.2}s duration",
            ch.name,
            ch.animation.joints.len(),
            ch.animation.duration
        );
    });

    let stop = Arc::new(AtomicBool::new(false));
    let shared_state = state::SharedState::new();
    let anim_graph = Arc::new(Mutex::new(anim::AnimGraph::default()));

    {
        let stop_flag = Arc::clone(&stop);
        ctrlc::set_handler(move || {
            println!("\nShutting down…");
            stop_flag.store(true, Ordering::Relaxed);
        })
        .expect("ctrlc handler");
    }

    server::start(&stop, &shared_state, &anim_graph);

    println!("REP on tcp://*:31337  |  PUB on tcp://*:31338");

    let mut behavior = behavior::BehaviorExecutive::new();
    let start = Instant::now();

    while !stop.load(Ordering::Relaxed) {
        let t = start.elapsed().as_secs_f64();
        anim_graph.lock().unwrap().update(t);
        behavior.update();
        std::thread::sleep(Duration::from_millis(10));
    }

    println!("Exited.");
}
