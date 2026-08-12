use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Pushes a client's local profile to the server for backup/cross-device use.
/// The profile payload is opaque JSON here — `deduced-protocol` doesn't know
/// or care about `deduced-save::Profile`'s exact shape, only that both sides
/// agree on *a* JSON document plus a logical clock (`updated_at`) to
/// reconcile with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSyncRequest {
    pub player_id: String,
    pub updated_at: u64,
    pub profile: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSyncResponse {
    pub updated_at: u64,
    pub profile: Value,
    /// `true` if the server stored the client's submitted profile (it was
    /// newer). `false` if the server already had an equal-or-newer profile —
    /// the client should adopt the returned one instead of its own.
    pub accepted: bool,
}
