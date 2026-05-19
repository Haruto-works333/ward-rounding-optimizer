# 職種制約つき病棟ラウンド最適化PoC

[English version](README.md)

職種制約を考慮した病棟ラウンド経路最適化のPoCです。

本プロジェクトでは、病棟ラウンドを組合せ最適化問題として扱います。
病室ごとのタスクを医師・看護師に割り当てる際に、職種制約、移動時間、タスク所要時間、点数、スタッフの稼働時間、同席制約を考慮します。

本プロジェクトでは実患者データや実際の病院運用データは使用していません。

## 現在の状態

Rust CLI として実装しています。現時点で揃っている要素は以下です。

- 病室、スタッフ、タスク、ルート、解を表すドメインモデル
- 固定の疑似シナリオ: `Mini Case 001`
- 2 種類のベースライン
  - `RoomOrderBaseline`
  - `DoctorAccompanyBaseline`
- greedy optimizer (best-insertion 配置)
- 違反種別を分類した制約検証
- ポイント / ペナルティ内訳つきスコア計算
- 看護師人数の感度分析
- clap ベースの CLI（サブコマンド + `--help`）
- 回帰テストと CLI スナップショットテスト

明示的に **未実装** のもの:

- DB 永続化
- API サーバー
- Web フロントエンド
- ランダムシナリオ生成
- local search / simulated annealing
- 実医療データ連携

## 問題設定

各タスクは病室に紐づき、以下の属性を持ちます。

- 必要職種
- 所要時間
- 点数
- 優先度

対応しているタスク要件は以下です。

- `DoctorRequired`
- `NurseCapable`
- `NurseOnly`
- `DoctorAndNurseSync`

主 KPI は以下です。

```text
points_per_doctor_minute = earned_points / doctor_active_minutes
```

希少リソースである医師の時間をどれだけ効率よく使えているかを表します。
`doctor_active_minutes` は拘束(オンデューティ)分で、計画ウィンドウ開始または医師の available_from のうち遅い方から、最後のコミットメント終了までの壁時計時間を指します。アイドル時間も含むので注意してください。

## リポジトリ構成

```text
ward-rounding-optimizer/
  apps/
    cli/              # CLI アプリケーション (clap ベース)
      tests/          # CLI スナップショットテスト

  crates/
    domain/           # コアドメイン型 (ロジックは持たない)
      geometry.rs     #   Point とマンハッタン距離
      ids.rs          #   RoomId, StaffId, TaskId の newtype
      problem.rs      #   ProblemInput
      room.rs / staff.rs / task.rs
      scoring.rs      #   ScoringWeights + ScoreBreakdown
      solution.rs     #   Visit, StaffRoute, Solution
      time.rs         #   Minute, TimeWindow

    optimizer/        # 配置戦略と評価ロジック
      solver.rs       #   Solver トレイトと unit-struct 戦略
      baseline.rs     #   RoomOrderBaseline
      baseline/
        doctor_accompany.rs
      greedy.rs       #   GreedyOptimizer (best-insertion)
      routing.rs      #   travel time / 最早開始時刻のヘルパー
      placement.rs    #   候補スコアリングと解の更新
      scoring.rs      #   score() 関数
      validate.rs     #   制約検証 (Violation)
      report.rs       #   MethodReport
      fixtures.rs     #   疑似シナリオ (mini_case_001)
```

## Solver API

すべての配置戦略は `optimizer::solver::Solver` を実装しています。

```rust
pub trait Solver {
    fn name(&self) -> &'static str;
    fn solve(&self, problem: &ProblemInput) -> Solution;
}
```

`optimizer::solver::all_solvers()` で同梱の全戦略を `Vec<Box<dyn Solver>>` として取得できます。直接インスタンス化したい場合は `RoomOrderBaseline` / `DoctorAccompanyBaseline` / `GreedyOptimizer` を使ってください。`greedy_optimizer(problem)` のような関数形式も利用可能です。

## 必要環境

- Rust
- Cargo

Rust が未インストールの場合は、rustup でインストールします。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## 使い方

サブコマンド一覧を表示します。

```bash
cargo run -p cli -- --help
```

Mini Case を実行します（サブコマンドを省略するとこれが既定）。

```bash
cargo run -p cli -- mini-case
```

ルート詳細および違反一覧つきで Mini Case を実行します。

```bash
cargo run -p cli -- mini-case --details
```

看護師人数の感度分析を実行します。

```bash
cargo run -p cli -- sensitivity --nurse-min 1 --nurse-max 4
```

## 出力例

```text
Scenario: mini-case-001
Planning window: 30 min
Doctor count: 1

Nurses | Method                   |  Score | Points | DoctorMin | DoctorTravel | NurseAvgMin |    Pts/DoctorMin | Unassigned | Violations
------------------------------------------------------------------------------------------------------------------------------------
2      | RoomOrderBaseline        |   2340 |   3250 |        30 |            4 |        27.5 |            108.3 |          1 |          0
2      | DoctorAccompanyBaseline  |   -621 |   1900 |        25 |            1 |        19.5 |             76.0 |          3 |          0
2      | GreedyOptimizer          |   2312 |   3250 |        28 |            7 |        27.0 |            116.1 |          1 |          0
```

## テスト

すべてのテストを実行します。

```bash
cargo test
```

最適化挙動、スコアリング重みの不変条件、`apps/cli/tests/` 配下の CLI スナップショットによる出力安定性をカバーしています。

## 今後の方向性

確定したロードマップは持ちません。考えられる発展先:

- SQLite 永続化
- Rust API サーバー
- Web フロントエンド
- より豊富なシナリオ生成
- local search などの最適化手法追加
- より現実的な運用制約への対応

## ライセンス

MIT
