/// How much of an answer's reveal art should be shown right now.
///
/// The renderer decides what a given level looks like (blur amount, tile count,
/// silhouette opacity, ...); this crate only tracks the deterministic progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealState {
    pub level: u8,
    pub max_level: u8,
}

/// `attempts_used` guesses have been made out of `max_attempts`. The level starts
/// at 1 (before any guess) and reaches `max_level` once the last attempt is used.
pub fn reveal_state(attempts_used: usize, max_attempts: usize) -> RevealState {
    let max_level = max_attempts.max(1);
    let level = (attempts_used + 1).min(max_level);

    RevealState {
        level: level as u8,
        max_level: max_level as u8,
    }
}
