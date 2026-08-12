use deduced_core::GameContent;
use sqlx::PgPool;

use crate::multiplayer::MultiplayerState;

pub struct AppState {
    pub content: GameContent,
    pub pool: PgPool,
    pub multiplayer: MultiplayerState,
}
