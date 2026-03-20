/// Jarvis Core — minimal FFI scaffold
/// Exposes a hello-world function via flutter_rust_bridge.

pub mod agents;
#[cfg(test)]
pub mod integration_tests;
pub mod audio;
pub mod brain;
pub mod coordination;
pub mod digital_twin;
pub mod execution;
pub mod infra;
pub mod intelligence;
pub mod integrations;
pub mod knowledge;
pub mod learning;
pub mod llm;
pub mod memory;
pub mod meta_cognition;
pub mod offline_validation;
pub mod pal;
pub mod perception;
pub mod personality;
pub mod pipeline;
pub mod planner;
pub mod proactive;
pub mod safety;
pub mod simulation;
pub mod sync;
pub mod vision;

/// Returns a greeting string. Used to verify FFI bridge is working.
pub fn greet(name: String) -> String {
    format!("Hello from Jarvis Core, {}!", name)
}

/// Returns the current version of the jarvis_core library.
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
