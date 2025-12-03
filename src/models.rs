use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/**
 * 각 리프트의 3RM 무게
 * 모든 무게는 f32로 관리
 */
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Lifts {
    pub bench_press: f32,
    pub squat: f32,
    pub deadlift: f32,
    pub overhead_press: f32,
}

/**
 * 사용자 정보
 * 이름, 3RM, 현재 주차 정보 포함
 */
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub name: String,
    /** Key: "{cycle}-{wave_reps}s" (e.g., "1-10s"), Value: Lifts for that wave */
    pub lift_history: HashMap<String, Lifts>,
    /** Key: "{cycle}-{wave_reps}s" (e.g., "1-10s"), Value: AMRAP reps for that wave */
    pub amrap_history: HashMap<String, AmrapReps>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct AmrapReps {
    pub bench_press: u8,
    pub squat: u8,
    pub deadlift: u8,
    pub overhead_press: u8,
}

/**
 * users.json 파일 전체의 데이터 구조
 * 사용자 이름을 key로 하여 User 데이터를 저장
 */
pub type UserData = HashMap<String, User>;

/**
 * 운동 한 세트의 정보
 * reps가 -1이면 AMRAP(As Many Reps As Possible)을 의미
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Set {
    pub weight: f32,
    pub reps: i8,
    pub percentage: f32,
}

/**
 * 하루의 운동 프로그램
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramDay {
    pub lift_name: String,
    pub sets: Vec<Set>,
}

/**
 * 한 주의 운동 프로그램
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramWeek {
    pub macro_week: u8,    // 1-16주
    pub wave_type: String, // "10s Wave", "8s Wave", ...
    pub week_in_wave: u8,  // 1-4주
    pub days: Vec<ProgramDay>,
}
