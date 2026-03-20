/// Infrastructure Layer
///
/// Cross-cutting infrastructure components:
/// - `connectivity` — DNS-based connectivity monitor emitting `ConnectivityStatus`
/// - `model_router` — routes LLM/STT/TTS/Vision workloads to local or cloud backends

pub mod connectivity;
pub mod model_router;
