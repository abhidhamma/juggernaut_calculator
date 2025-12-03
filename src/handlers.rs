use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::env;

use crate::{
    // 'crate::'를 사용하여 라이브러리 루트에서 모듈을 가져옴
    logic::{calculate_new_3rm, generate_single_week_program, LiftType},
    models::{AmrapReps, Lifts, ProgramWeek, User},
};

/** Upstash API와 통신하기 위한 HTTP 클라이언트를 생성 */
fn kv_client() -> Result<reqwest::Client, StatusCode> {
    let token = env::var("KV_REST_API_TOKEN").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/**
 * 특정 사용자의 정보를 가져오는 핸들러
 * /api/user/{username}
 */
pub async fn get_user(Path(username): Path<String>) -> Result<Json<User>, StatusCode> {
    let kv_url = env::var("KV_REST_API_URL").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let client = kv_client()?;
    let key = format!("user:{}", username);

    let res = client
        .get(format!("{}/get/{}", kv_url, key))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(Deserialize)]
    struct KvResponse {
        result: Option<String>,
    }
    let body: KvResponse = res
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(user_str) = body.result {
        let user: User =
            serde_json::from_str(&user_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(user))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Deserialize)]
pub struct UpsertPayload {
    #[serde(rename = "waveKey")]
    wave_key: String,
    lifts: Lifts,
}

/**
 * 특정 웨이브의 3RM 정보를 생성하거나 업데이트하는 핸들러
 * POST /api/user/{username}
 */
pub async fn upsert_user(
    Path(username): Path<String>,
    Json(payload): Json<UpsertPayload>,
) -> Result<Json<User>, StatusCode> {
    let kv_url = env::var("KV_REST_API_URL").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let client = kv_client()?;
    let key = format!("user:{}", username);

    // 1. KV에서 직접 사용자 정보를 읽어옴
    let res = client
        .get(format!("{}/get/{}", kv_url, key))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(Deserialize)]
    struct KvResponse {
        result: Option<String>,
    }
    let body: KvResponse = res
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. 사용자 정보가 있으면 파싱하고, 없으면 새로 생성
    let mut user: User = if let Some(user_str) = body.result {
        serde_json::from_str(&user_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        User {
            name: username.clone(),
            ..Default::default()
        }
    };

    // 3. 받은 정보로 lift_history 업데이트
    user.lift_history.insert(payload.wave_key, payload.lifts);

    // 4. 업데이트된 전체 사용자 객체를 Upstash에 저장
    let user_json_string =
        serde_json::to_string(&user).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    client
        .post(format!("{}/set/{}", kv_url, key))
        .body(user_json_string)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

#[derive(Deserialize)]
pub struct ProgramQuery {
    cycle: u32,
    wave: u8, // 10, 8, 5, 3
    week: u8, // 1, 2, 3, 4
}

/**
 * 쿼리 파라미터에 맞는 특정 주의 프로그램을 생성하여 반환
 * GET /api/program/{username}
 */
pub async fn get_program(
    Path(username): Path<String>,
    Query(query): Query<ProgramQuery>,
) -> Result<Json<ProgramWeek>, StatusCode> {
    let user = get_user(Path(username)).await?.0;

    // e.g., "1-10s"
    let wave_key = format!("{}-{}s", query.cycle, query.wave);

    // 해당 웨이브의 3RM 정보를 찾음. 없으면 404.
    let lifts = user.lift_history.get(&wave_key).ok_or_else(|| {
        println!("Lifts not found for key: {}", wave_key);
        StatusCode::NOT_FOUND
    })?;

    let program = generate_single_week_program(lifts, query.wave, query.week);
    Ok(Json(program))
}

#[derive(Deserialize, Debug)]
pub struct AmrapPayload {
    cycle: u32,
    wave: u8, // 현재 wave (e.g., 10)
    // 각 리프트별 AMRAP 성공 횟수
    amrap_reps: AmrapReps,
}

/**
 * AMRAP 결과를 받아 다음 Wave의 3RM을 계산하고 저장
 */
pub async fn calculate_and_save_amrap(
    Path(username): Path<String>,
    Json(payload): Json<AmrapPayload>,
) -> Result<Json<User>, StatusCode> {
    // 1. 기존 사용자 정보 가져오기
    let mut user = get_user(Path(username.clone())).await?.0;

    let current_wave_key = format!("{}-{}s", payload.cycle, payload.wave);
    let current_lifts = user
        .lift_history
        .get(&current_wave_key)
        .cloned() // 다음 계산을 위해 값을 복제
        .ok_or(StatusCode::BAD_REQUEST)?;

    let next_wave_reps = match payload.wave {
        10 => 8,
        8 => 5,
        5 => 3,
        3 => 10, // 다음 사이클의 10s wave
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let next_cycle = if payload.wave == 3 {
        payload.cycle + 1
    } else {
        payload.cycle
    };
    let next_wave_key = format!("{}-{}s", next_cycle, next_wave_reps);

    let new_lifts = Lifts {
        bench_press: calculate_new_3rm(
            current_lifts.bench_press,
            &LiftType::UpperBody,
            payload.wave,
            payload.amrap_reps.bench_press,
        ),
        squat: calculate_new_3rm(
            current_lifts.squat,
            &LiftType::LowerBody,
            payload.wave,
            payload.amrap_reps.squat,
        ),
        deadlift: calculate_new_3rm(
            current_lifts.deadlift,
            &LiftType::LowerBody,
            payload.wave,
            payload.amrap_reps.deadlift,
        ),
        overhead_press: calculate_new_3rm(
            current_lifts.overhead_press,
            &LiftType::UpperBody,
            payload.wave,
            payload.amrap_reps.overhead_press,
        ),
    };

    // 계산된 새 3RM을 다음 wave의 키로 저장
    user.lift_history.insert(next_wave_key, new_lifts);

    // 이번 wave의 AMRAP 결과도 저장
    user.amrap_history
        .insert(current_wave_key, payload.amrap_reps);

    // 업데이트된 전체 사용자 객체를 Upstash에 저장
    let kv_url = env::var("KV_REST_API_URL").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let client = kv_client()?;
    let key = format!("user:{}", username);
    let user_json_string =
        serde_json::to_string(&user).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    client
        .post(format!("{}/set/{}", kv_url, key))
        .body(user_json_string)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}
