use axum::{
    routing::{get, get_service, post},
    Router,
};
use std::net::SocketAddr;

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

    // Vercel 라우팅 규칙에 따라 API 라우터만 정의
    let app = api_router;

    // 서버 실행
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("서버 실행 중: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
