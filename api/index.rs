use axum::{
    routing::{get, get_service, post},
    Router,
};
use juggernaut_calculator::handlers::{
    calculate_and_save_amrap, get_program, get_user, upsert_user,
};
use once_cell::sync::Lazy;
use tower_http::services::ServeDir;
use vercel_runtime::{run, Body, Error, Request, Response};

/**
 * Axum 라우터를 한 번만 생성하여 재사용하기 위한 static 변수
 * Lazy를 사용하여 처음 호출될 때 라우터가 초기화됨
 */
static ROUTER: Lazy<Router> = Lazy::new(|| {
    Router::new()
        .route("/api/user/:username", get(get_user).post(upsert_user))
        .route("/api/program/:username", get(get_program))
        .route("/api/amrap/:username", post(calculate_and_save_amrap))
        .fallback_service(get_service(ServeDir::new("public")))
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    run(handler).await?;
    Ok(())
}

async fn handler(req: Request) -> Result<Response<Body>, Error> {
    Ok(ROUTER.clone().call(req).await?)
}
