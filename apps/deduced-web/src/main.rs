mod api;
mod state;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use deduced_content::load_content_from_dir;

use state::AppState;

#[tokio::main]
async fn main() {
    let content = load_content_from_dir(Path::new("content"))
        .expect("game content must load from the content/ directory");

    let state = Arc::new(AppState::new(content));

    let static_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("static");
    let serve_dir =
        ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html")));

    let app = Router::new()
        .merge(api::routes())
        .fallback_service(serve_dir)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4173));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");

    println!("DEDUCED web running at http://{addr}");
    axum::serve(listener, app).await.expect("server error");
}
