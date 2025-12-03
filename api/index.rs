use axum::{
    routing::{get, get_service, post},
    Router,
};
use juggernaut_calculator::handlers::{
    calculate_and_save_amrap, get_program, get_user, upsert_user,
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/api/user/:username", get(get_user).post(upsert_user))
        .route("/api/program/:username", get(get_program))
        .route("/api/amrap/:username", post(calculate_and_save_amrap))
        .fallback_service(get_service(ServeDir::new("public")));

    vercel_axum::run(app).await;
    Ok(())
}
