use crate::models::{Lifts, ProgramDay, ProgramWeek, Set};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LiftType {
    UpperBody,
    LowerBody,
}

/**
 * Working Max(WM) 계산: 3RM * 0.9
 */
fn calculate_wm(rm3: f32) -> f32 {
    rm3 * 0.9
}

/**
 * 실제 운동 무게 계산 (가장 가까운 2.5kg 단위로 반올림)
 */
fn round_to_nearest_2_5(weight: f32) -> f32 {
    (weight / 2.5).round() * 2.5
}

/**
 * 주어진 주차와 웨이브에 맞는 세트/반복/강도 생성
 */
fn get_sets(week_in_wave: u8, wave_reps: u8, wm: f32) -> Vec<Set> {
    let (percents, reps, num_sets) = match (week_in_wave, wave_reps) {
        // Accumulation Week
        (1, 10) => (vec![0.60], vec![10], 5),
        (1, 8) => (vec![0.65], vec![8], 5),
        (1, 5) => (vec![0.70], vec![5], 5),
        (1, 3) => (vec![0.75], vec![3], 5),
        // Intensification Week
        (2, 10) => (vec![0.50, 0.60, 0.70], vec![5, 5, 10], 3),
        (2, 8) => (vec![0.55, 0.65, 0.75], vec![5, 5, 8], 3),
        (2, 5) => (vec![0.60, 0.70, 0.80], vec![5, 3, 5], 3),
        (2, 3) => (vec![0.65, 0.75, 0.85], vec![3, 3, 3], 3),
        // Realization Week
        (3, 10) => (vec![0.50, 0.60, 0.70, 0.75], vec![5, 3, 1, -1], 4), // -1 for AMRAP
        (3, 8) => (vec![0.55, 0.65, 0.75, 0.80], vec![5, 3, 1, -1], 4),
        (3, 5) => (vec![0.60, 0.70, 0.80, 0.85], vec![3, 2, 1, -1], 4),
        (3, 3) => (vec![0.65, 0.75, 0.85, 0.90], vec![3, 1, 1, -1], 4),
        // Deload Week
        (4, _) => (vec![0.40, 0.50, 0.60], vec![5, 5, 5], 3),
        _ => (vec![], vec![], 0),
    };

    let mut sets = Vec::new();
    if week_in_wave == 2 || week_in_wave == 3 || week_in_wave == 4 {
        // 여러 강도를 사용하는 주
        for i in 0..num_sets {
            sets.push(Set {
                weight: round_to_nearest_2_5(wm * percents[i]),
                reps: reps[i],
                percentage: percents[i],
            });
        }
    } else {
        // 단일 강도를 사용하는 주
        for _ in 0..num_sets {
            sets.push(Set {
                weight: round_to_nearest_2_5(wm * percents[0]),
                reps: reps[0],
                percentage: percents[0],
            });
        }
    }
    sets
}

/**
 * 특정 주차의 프로그램을 생성하는 함수
 */
pub fn generate_single_week_program(lifts: &Lifts, wave_reps: u8, week_in_wave: u8) -> ProgramWeek {
    let lift_wms = [
        calculate_wm(lifts.bench_press),
        calculate_wm(lifts.squat),
        calculate_wm(lifts.deadlift),
        calculate_wm(lifts.overhead_press),
    ];
    let lift_names = ["Bench Press", "Squat", "Deadlift", "Overhead Press"];

    let mut days: Vec<ProgramDay> = Vec::new();
    for (lift_idx, &wm) in lift_wms.iter().enumerate() {
        days.push(ProgramDay {
            lift_name: lift_names[lift_idx].to_string(),
            sets: get_sets(week_in_wave, wave_reps, wm),
        });
    }

    ProgramWeek {
        macro_week: 0, // 이 필드는 이제 덜 중요해짐
        wave_type: format!("{}s Wave", wave_reps),
        week_in_wave,
        days,
    }
}

/**
 * AMRAP 수행 결과에 따라 다음 Wave의 3RM을 계산
 */
pub fn calculate_new_3rm(
    current_3rm: f32,
    lift_type: &LiftType,
    target_reps: u8,
    actual_reps: u8,
) -> f32 {
    let reps_over_target = actual_reps.saturating_sub(target_reps);
    if reps_over_target == 0 {
        return current_3rm; // 목표 횟수를 넘지 못하면 증량 없음
    }

    let current_wm = calculate_wm(current_3rm);

    // 점진적 과부하 규칙에 따른 WM 증가량 계산
    let wm_increase = match lift_type {
        LiftType::UpperBody => {
            // 상체: 초과 1회당 0.5kg (WM 기준)
            // 이 부분은 논의가 필요하지만, 보통 증량은 TM/WM에 직접 더함
            // 여기서는 사용자 요청에 따라 단순화된 공식을 적용
            reps_over_target as f32 * 0.5
        }
        LiftType::LowerBody => {
            // 하체: 초과 1회당 1.25kg (WM 기준)
            reps_over_target as f32 * 1.25
        }
    };

    let new_wm = current_wm + wm_increase;

    // 새로운 WM을 기반으로 새로운 3RM 역산
    let new_3rm = new_wm / 0.9;

    // 소수점 두 자리에서 반올림
    (new_3rm * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Lifts;

    // 테스트에 사용할 기본 3RM 값
    fn get_test_lifts() -> Lifts {
        Lifts {
            bench_press: 100.0,
            squat: 140.0,
            deadlift: 180.0,
            overhead_press: 60.0,
        }
    }

    #[test]
    fn test_10s_wave_accumulation_week() {
        let lifts = get_test_lifts();
        let program = generate_single_week_program(&lifts, 10, 1); // 10s wave, 1주차

        assert_eq!(program.wave_type, "10s Wave");
        assert_eq!(program.week_in_wave, 1);

        // 벤치프레스(3RM 100kg -> WM 90kg) 테스트
        let bench_day = &program.days[0];
        assert_eq!(bench_day.lift_name, "Bench Press");
        assert_eq!(bench_day.sets.len(), 5); // 5세트
                                             // 60% of WM = 90 * 0.6 = 54.0 -> rounded to 55.0
        assert_eq!(bench_day.sets[0].weight, 55.0);
        assert_eq!(bench_day.sets[0].reps, 10); // 10회
    }

    #[test]
    fn test_10s_wave_intensification_week() {
        let lifts = get_test_lifts();
        let program = generate_single_week_program(&lifts, 10, 2); // 10s wave, 2주차

        let bench_day = &program.days[0];
        assert_eq!(bench_day.sets.len(), 3);
        // 70% of WM = 90 * 0.7 = 63.0 -> rounded to 62.5
        assert_eq!(bench_day.sets[2].weight, 62.5);
        assert_eq!(bench_day.sets[2].reps, 10);
    }

    #[test]
    fn test_10s_wave_realization_week() {
        let lifts = get_test_lifts();
        let program = generate_single_week_program(&lifts, 10, 3); // 10s wave, 3주차

        let bench_day = &program.days[0];
        assert_eq!(bench_day.sets.len(), 4);
        // AMRAP set: 75% of WM = 90 * 0.75 = 67.5
        let amrap_set = &bench_day.sets[3];
        assert_eq!(amrap_set.weight, 67.5);
        assert_eq!(amrap_set.reps, -1); // -1은 AMRAP을 의미
    }

    #[test]
    fn test_10s_wave_deload_week() {
        let lifts = get_test_lifts();
        let program = generate_single_week_program(&lifts, 10, 4); // 10s wave, 4주차

        let bench_day = &program.days[0];
        assert_eq!(bench_day.sets.len(), 3);
        // 60% of WM = 90 * 0.6 = 54.0 -> rounded to 55.0
        assert_eq!(bench_day.sets[2].weight, 55.0);
        assert_eq!(bench_day.sets[2].reps, 5);
    }

    #[test]
    fn test_calculate_new_3rm_upper_body() {
        // 10s wave (목표 10회), 12회 성공 시
        // 초과 2회 * 0.5kg = 1kg WM 증가
        // 기존 3RM 100 -> WM 90 -> new WM 91 -> new 3RM 101.11...
        let new_rm = calculate_new_3rm(100.0, &LiftType::UpperBody, 10, 12);
        assert_eq!(new_rm, 101.11);
    }

    #[test]
    fn test_calculate_new_3rm_lower_body() {
        // 8s wave (목표 8회), 10회 성공 시
        // 초과 2회 * 1.25kg = 2.5kg WM 증가
        // 기존 3RM 140 -> WM 126 -> new WM 128.5 -> new 3RM 142.77...
        let new_rm = calculate_new_3rm(140.0, &LiftType::LowerBody, 8, 10);
        assert_eq!(new_rm, 142.78);
    }
}
