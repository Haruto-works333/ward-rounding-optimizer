//! Scoring inputs and outputs.
//!
//! `ScoringWeights` is the *configuration* fed into the score function (how much each
//! cost / penalty is worth), and `ScoreBreakdown` is the structured *result* of scoring
//! a solution against those weights. They live together because every consumer of one
//! tends to consume the other.
//!
//! スコアリングの入出力。`ScoringWeights` はスコア関数への *設定* (各コスト・ペナルティの
//! 単価) で、`ScoreBreakdown` はその重みのもとで解をスコアリングした *結果* の構造体。
//! 片方を使う側はもう片方も使うので、同居させている。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoringWeights {
    pub doctor_work_minute_penalty: i32,
    pub doctor_travel_minute_penalty: i32,
    pub nurse_work_minute_penalty: i32,
    pub nurse_travel_minute_penalty: i32,
    pub unassigned_high_priority_penalty: i32,
    pub unassigned_normal_priority_penalty: i32,
    pub unassigned_low_priority_penalty: i32,
}

impl Default for ScoringWeights {
    /// Placeholder weights. Doctor minutes are penalized ~6x nurse minutes to reflect
    /// the doctor being the scarce resource, and travel is penalized at roughly half
    /// the work rate so routes are discouraged from moving without producing value.
    /// Unassigned-task penalties dominate the per-minute costs so the optimizer prefers
    /// scheduling a task over leaving it on the table. The structural inequalities
    /// between these numbers are pinned by `default_scoring_weights_keep_design_inequalities`.
    ///
    /// 暫定重み。希少リソースである医師の1分は看護師の約6倍にペナルティ付けされ、
    /// travel は work のおよそ半分の重みにすることで価値を生まない移動を抑制する。
    /// 未割当タスクのペナルティは per-minute のコストよりも十分大きく、optimizer は
    /// 基本的にタスクを残すよりも割り当てる方を選ぶ。各重み間の大小関係は
    /// `default_scoring_weights_keep_design_inequalities` テストで固定している。
    fn default() -> Self {
        Self {
            doctor_work_minute_penalty: 20,
            doctor_travel_minute_penalty: 10,
            nurse_work_minute_penalty: 3,
            nurse_travel_minute_penalty: 2,
            unassigned_high_priority_penalty: 1000,
            unassigned_normal_priority_penalty: 300,
            unassigned_low_priority_penalty: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreBreakdown {
    pub total_score: i32,

    pub earned_points: i32,

    pub doctor_work_minutes: i32,
    pub doctor_travel_minutes: i32,
    pub nurse_work_minutes: i32,
    pub nurse_travel_minutes: i32,

    /// On-duty (engagement) minutes per role.
    ///
    /// Measured as `latest_commitment_end - max(planning_window.start, staff.available_from)`,
    /// where `latest_commitment_end` is the later of the last visit's end and
    /// `forced_active_until` (e.g. nurse accompanying a doctor). This counts the wall-clock
    /// span the staff is committed to the schedule, including idle gaps and pre-first-visit
    /// idle time, NOT just active work minutes. It is the denominator of
    /// `points_per_doctor_minute`.
    ///
    /// ロール別の拘束(オンデューティ)分。`latest_commitment_end -
    /// max(planning_window.start, staff.available_from)` で計算する。
    /// `latest_commitment_end` は最後の visit 終了と `forced_active_until`
    /// (医師に同行する看護師など) の大きい方。実稼働分ではなく、スケジュールに
    /// 拘束される壁時計上の時間で、最初の visit 前のアイドルや visit 間の
    /// 空き時間も含む。`points_per_doctor_minute` の分母として用いる。
    pub doctor_active_minutes: i32,
    pub nurse_total_active_minutes: i32,
    pub nurse_avg_active_minutes: f64,

    pub points_per_doctor_minute: f64,

    pub unassigned_task_count: usize,
    pub unassigned_high_priority_count: usize,
    pub unassigned_normal_priority_count: usize,
    pub unassigned_low_priority_count: usize,
}

impl ScoreBreakdown {
    pub fn zero() -> Self {
        Self {
            total_score: 0,
            earned_points: 0,

            doctor_work_minutes: 0,
            doctor_travel_minutes: 0,
            nurse_work_minutes: 0,
            nurse_travel_minutes: 0,

            doctor_active_minutes: 0,
            nurse_total_active_minutes: 0,
            nurse_avg_active_minutes: 0.0,

            points_per_doctor_minute: 0.0,

            unassigned_task_count: 0,
            unassigned_high_priority_count: 0,
            unassigned_normal_priority_count: 0,
            unassigned_low_priority_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pin down the structural relationships of the default scoring weights so an accidental
    /// swap (e.g. doctor/nurse penalties reversed) cannot pass review silently. The exact
    /// numbers are placeholders subject to tuning, but the *ordering* between them encodes
    /// the design intent and must stay stable.
    ///
    /// デフォルト重みの構造関係を固定するテスト。具体的な数値はチューニング対象の
    /// 暫定値だが、重み間の大小関係が設計意図(医師>看護師、work>travel、
    /// 未割当>per-minute)を表現しており、誤って入れ替わると最適化方向が崩れるため、
    /// ここで検知する。
    #[test]
    fn default_scoring_weights_keep_design_inequalities() {
        let weights = ScoringWeights::default();

        // Doctor minutes are the scarcer resource — penalize them harder than nurse minutes.
        // 医師の時間は希少リソース。看護師より強くペナルティを掛ける。
        assert!(
            weights.doctor_work_minute_penalty > weights.nurse_work_minute_penalty,
            "doctor work penalty must exceed nurse work penalty"
        );
        assert!(
            weights.doctor_travel_minute_penalty > weights.nurse_travel_minute_penalty,
            "doctor travel penalty must exceed nurse travel penalty"
        );

        // Travel is value-less motion — penalize less than work so the optimizer doesn't
        // refuse to move at all, but enough to discourage detours.
        // travel は価値を生まない移動なので work より軽く、ただしゼロにはしない。
        assert!(
            weights.doctor_travel_minute_penalty < weights.doctor_work_minute_penalty,
            "doctor travel penalty must be lighter than doctor work penalty"
        );
        assert!(
            weights.nurse_travel_minute_penalty < weights.nurse_work_minute_penalty,
            "nurse travel penalty must be lighter than nurse work penalty"
        );
        assert!(weights.doctor_travel_minute_penalty > 0);
        assert!(weights.nurse_travel_minute_penalty > 0);

        // Priority ordering: High > Normal > Low. Leaving a high-priority task unassigned
        // must cost more than leaving a normal one, which must cost more than a low one.
        // 優先度の順序 High > Normal > Low。未割当ペナルティもこの順で大きくする。
        assert!(
            weights.unassigned_high_priority_penalty > weights.unassigned_normal_priority_penalty,
            "high-priority unassigned penalty must exceed normal-priority"
        );
        assert!(
            weights.unassigned_normal_priority_penalty > weights.unassigned_low_priority_penalty,
            "normal-priority unassigned penalty must exceed low-priority"
        );

        // Unassigned penalties must dominate per-minute costs, otherwise the optimizer
        // prefers leaving a task on the table over spending a few doctor minutes on it.
        // 未割当ペナルティは per-minute コストより十分大きく。さもないと「割り当てるより
        // 残す方が安い」となり optimizer がタスクを残してしまう。
        assert!(
            weights.unassigned_low_priority_penalty > weights.doctor_work_minute_penalty,
            "even low-priority unassigned penalty must exceed doctor work minute penalty"
        );
    }
}
