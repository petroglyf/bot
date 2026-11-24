//! `ZeroMQ` server: exposes the SMURFS RPC API on a REP socket (port 31337)
//! and publishes 60 Hz joint-position frames on a PUB socket (port 31338).
//!
//! Wire encoding is `FlatBuffers` (`rpc.fbs`).

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use flatbuffers::FlatBufferBuilder;

use crate::anim::AnimGraph;
use crate::data_pool;
use crate::state::SharedState;

// Include flatc-generated code.
#[allow(warnings, clippy::all, clippy::pedantic)]
mod gen {
    include!("gen/rpc_generated.rs");
}
use gen::smurfs::{
    root_as_msg_rpc, FuncCall, MsgRpc, MsgRpcArgs, RawJoints, RawJointsArgs, Vec3 as FbVec3,
};

const REP_PORT: u16 = 31337;
const PUB_PORT: u16 = 31338;

/// Spawn the REP and PUB server threads.
///
/// Both threads run until `stop` is set to `true`.
pub fn start(stop: &Arc<AtomicBool>, state: &SharedState, anim_graph: &Arc<Mutex<AnimGraph>>) {
    let stop_rep = Arc::clone(stop);
    let state_rep = state.clone();
    let anim_rep = Arc::clone(anim_graph);

    thread::spawn(move || run_rpc(stop_rep, state_rep, anim_rep));

    let stop_pub = Arc::clone(stop);
    let anim_pub = Arc::clone(anim_graph);
    thread::spawn(move || run_stream(stop_pub, anim_pub));
}

/// REP socket loop: receives `MsgRpc` `FlatBuffers` messages from clients and
/// dispatches to the appropriate handler.
// The thread owns these handles for its whole lifetime, so by-value is intended.
#[allow(clippy::needless_pass_by_value)]
fn run_rpc(stop: Arc<AtomicBool>, state: SharedState, anim_graph: Arc<Mutex<AnimGraph>>) {
    let ctx = zmq::Context::new();
    let sock = ctx.socket(zmq::REP).expect("zmq REP socket");
    sock.bind(&format!("tcp://*:{REP_PORT}")).expect("bind REP");
    sock.set_rcvtimeo(100).expect("set_rcvtimeo");

    while !stop.load(Ordering::Relaxed) {
        let msg = match sock.recv_bytes(0) {
            Ok(b) => b,
            Err(zmq::Error::EAGAIN) => continue, // timeout, check stop flag
            Err(e) => {
                eprintln!("REP recv error: {e}");
                break;
            }
        };

        let reply = dispatch(&msg, &state, &anim_graph);
        if let Err(e) = sock.send(&reply, 0) {
            eprintln!("REP send error: {e}");
        }
    }
}

/// PUB socket loop: broadcasts a `RawJoints` `FlatBuffer` at ~60 Hz.
///
/// Topic byte `0x00` = `JOINT_WATCHING`.
// The thread owns these handles for its whole lifetime, so by-value is intended.
#[allow(clippy::needless_pass_by_value)]
fn run_stream(stop: Arc<AtomicBool>, anim_graph: Arc<Mutex<AnimGraph>>) {
    let ctx = zmq::Context::new();
    let sock = ctx.socket(zmq::PUB).expect("zmq PUB socket");
    sock.bind(&format!("tcp://*:{PUB_PORT}")).expect("bind PUB");

    while !stop.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(16)); // ~60 Hz

        let pose: Vec<f64> = anim_graph.lock().unwrap().pose.clone();
        if pose.is_empty() {
            continue;
        }

        let frame = build_raw_joints(&pose);

        // ZMQ multi-part: topic byte then payload
        if sock.send(&[0x00u8][..], zmq::SNDMORE).is_ok() {
            let _ = sock.send(frame, 0);
        }
    }
}

/// Decode a `MsgRpc` `FlatBuffer` and route to the correct handler.
fn dispatch(msg: &[u8], state: &SharedState, _anim_graph: &Arc<Mutex<AnimGraph>>) -> Vec<u8> {
    let Ok(rpc_msg) = root_as_msg_rpc(msg) else {
        return build_string_reply("error: invalid flatbuffer");
    };

    let Some(call) = rpc_msg.call() else {
        return build_string_reply("error: missing call");
    };

    match call.call_id() {
        FuncCall::GetCharName => {
            // Returns the loaded character name.
            let name =
                data_pool::with_state(|ch| ch.name.clone()).unwrap_or_else(|| "none".to_string());
            build_string_reply(&name)
        }

        FuncCall::GetMotorJoints => build_joint_ids(),

        FuncCall::LoadChar => {
            // Hot-reloads a character from a path supplied as a_string.
            let path = call.a_string().unwrap_or("nao.glb");
            match data_pool::load(std::path::Path::new(path)) {
                Ok(()) => build_string_reply("ok"),
                Err(e) => build_string_reply(&format!("error: {e}")),
            }
        }

        FuncCall::LookAt => handle_vec3(call, state, "LookAt"),

        FuncCall::PointAt => handle_vec3(call, state, "PointAt"),

        // Clock control is handled in the main loop, and Connect/Disconnect/
        // StreamSpec are best-effort with PUB; acknowledge only.
        FuncCall::StartAnim
        | FuncCall::StopAnim
        | FuncCall::Disconnect
        | FuncCall::Connect
        | FuncCall::StreamSpec => build_string_reply("ok"),

        _ => build_string_reply("error: unknown call"),
    }
}

// ── FlatBuffer builders ────────────────────────────────────────────────────

fn build_string_reply(s: &str) -> Vec<u8> {
    let mut b = FlatBufferBuilder::new();
    let str_off = b.create_string(s);
    let msg = MsgRpc::create(
        &mut b,
        &MsgRpcArgs {
            a_string: Some(str_off),
            ..Default::default()
        },
    );
    b.finish(msg, None);
    b.finished_data().to_vec()
}

fn build_joint_ids() -> Vec<u8> {
    use gen::smurfs::{JointId, JointIdArgs, JointIds, JointIdsArgs};

    let mut b = FlatBufferBuilder::new();

    let ids: Vec<_> = data_pool::with_state(|ch| {
        ch.animation
            .joints
            .iter()
            .enumerate()
            .map(|(i, j)| {
                let name = b.create_string(&j.name);
                JointId::create(
                    &mut b,
                    &JointIdArgs {
                        name: Some(name),
                        id: i32::try_from(i).unwrap_or(i32::MAX),
                    },
                )
            })
            .collect()
    })
    .unwrap_or_default();

    let ids_vec = b.create_vector(&ids);
    let jids = JointIds::create(&mut b, &JointIdsArgs { ids: Some(ids_vec) });
    let msg = MsgRpc::create(
        &mut b,
        &MsgRpcArgs {
            joint_ids: Some(jids),
            ..Default::default()
        },
    );
    b.finish(msg, None);
    b.finished_data().to_vec()
}

fn build_raw_joints(pose: &[f64]) -> Vec<u8> {
    use gen::smurfs::{Joint, JointArgs};

    let mut b = FlatBufferBuilder::new();

    let joints: Vec<_> = pose
        .iter()
        .enumerate()
        .map(|(i, &rpos)| {
            Joint::create(
                &mut b,
                &JointArgs {
                    idx: i32::try_from(i).unwrap_or(i32::MAX),
                    rpos,
                },
            )
        })
        .collect();

    let joints_vec = b.create_vector(&joints);
    let raw = RawJoints::create(
        &mut b,
        &RawJointsArgs {
            joints: Some(joints_vec),
        },
    );
    let msg = MsgRpc::create(
        &mut b,
        &MsgRpcArgs {
            joints: Some(raw),
            ..Default::default()
        },
    );
    b.finish(msg, None);
    b.finished_data().to_vec()
}

fn handle_vec3(call: gen::smurfs::Rpc<'_>, state: &SharedState, key: &str) -> Vec<u8> {
    if let Some(v) = call.vec3() {
        state.set(key, [v.x(), v.y(), v.z()]);
    }
    let [x, y, z] = state.get(key).unwrap_or([0.0; 3]);

    let v3 = FbVec3::new(x, y, z);
    let mut b = FlatBufferBuilder::new();
    let msg = MsgRpc::create(
        &mut b,
        &MsgRpcArgs {
            vec3: Some(&v3),
            ..Default::default()
        },
    );
    b.finish(msg, None);
    b.finished_data().to_vec()
}
