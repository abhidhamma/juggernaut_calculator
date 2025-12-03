use axum::{
    routing::{get, get_service, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

use juggernaut_calculator::handlers::{
    calculate_and_save_amrap, get_program, get_user, upsert_user,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // .env 파일에서 환경 변수 로드
    dotenvy::dotenv().ok();

    // API 라우터를 먼저 정의
    let api_router = Router::new()
        .route("/api/user/:username", get(get_user).post(upsert_user)) // API 라우트
        .route("/api/program/:username", get(get_program))
        .route("/api/amrap/:username", post(calculate_and_save_amrap));

    // API 라우터를 먼저 시도하고, 없으면 정적 파일 서빙으로 fallback
    let app = api_router.fallback_service(get_service(ServeDir::new("public")));

    // 서버 실행
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("서버 실행 중: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
