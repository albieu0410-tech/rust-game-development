use std::time::{SystemTime, UNIX_EPOCH};

use deduced_protocol::ProfileSyncRequest;
use deduced_save::Profile;

/// Best-effort, fire-and-forget push of the local profile to the server.
///
/// This never blocks the game loop and never fails visibly: Solo must stay
/// fully playable with the server unreachable, so any error here is just
/// logged to the console and otherwise ignored.
pub fn sync_profile_in_background(profile: &Profile) {
    let updated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let Ok(profile_json) = serde_json::to_value(profile) else {
        return;
    };

    let request = ProfileSyncRequest {
        player_id: profile.player_id.clone(),
        updated_at,
        profile: profile_json,
    };

    std::thread::spawn(move || {
        let url = format!("{}/profile/sync", crate::server::BASE_URL);
        let result = ureq::post(&url).send_json(&request);
        if let Err(err) = result {
            eprintln!("profile sync skipped (server unreachable?): {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_never_panics_or_blocks_when_the_server_is_unreachable() {
        // No server is running on this port in the test environment. The call
        // must return immediately (the request runs on a detached thread) and
        // must not panic — Solo has to stay playable regardless of network state.
        sync_profile_in_background(&Profile::default());
    }
}
