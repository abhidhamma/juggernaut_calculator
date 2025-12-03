use axum::{
    routing::{get, get_service, post},
    Router,
};
use juggernaut_calculator::handlers::{
    calculate_and_save_amrap, get_program, get_user, upsert_user,
};
use tower_http::services::ServeDir;
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/api/user/:username", get(get_user).post(upsert_user))
        .route("/api/program/:username", get(get_program))
        .route("/api/amrap/:username", post(calculate_and_save_amrap))
        /** API가 아닌 모든 요청을 public 폴더에서 처리하도록 fallback 설정 */
        .fallback_service(get_service(ServeDir::new("public")));

    run(app).await?;
    Ok(())
}

/** 이 핸들러는 vercel_runtime::run(app)을 사용하므로 실제로는 호출되지 않음 */
async fn handler(req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder().status(200).body(Body::Empty)?)
}
