use axum::{
    routing::{get, post},
    Router,
};
use juggernaut_calculator::handlers::{
    calculate_and_save_amrap, get_program, get_user, upsert_user,
};
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/api/user/:username", get(get_user).post(upsert_user)) // API 라우트
        .route("/api/program/:username", get(get_program))
        .route("/api/amrap/:username", post(calculate_and_save_amrap));

    run(app).await?;
    Ok(())
}

async fn handler(req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder().status(200).body(Body::Empty)?)
}
