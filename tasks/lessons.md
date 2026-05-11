# tasks/lessons.md

このファイルは、ユーザーからの修正や失敗から得た再発防止ルールを記録する。

## Rules

- ルールは具体的に書く（「何を」「いつ」「どう防ぐか」）
- 再発した場合はルールを更新して、曖昧な文言を削る
- 直近の作業開始前に必ず読み返す

## Entries

### 2026-02-27

- Context: Markdown Lint を `npm run lint:md` で実行した際、ローカルに `markdownlint` が存在しなかった。
- Mistake: コマンド失敗後の代替手順を標準化していなかった。
- Rule: ツールが未インストールの環境では `npx --yes <tool>` で即時フォールバックする。
- Checkpoint: `npx --yes markdownlint-cli CLAUDE.md tasks/*.md --config .markdownlint.json --ignore-path .markdownlintignore`

### 2026-03-10

- Context: `gwt-spec` ラベル付き Issue #107 の実装後、Issue 本文の `Tasks` が未更新のまま「残り 2 点」と判断してしまった。
- Mistake: 完了判定の一次情報を Issue 本文ではなく、自分の縮約した内部チェックリストに置いてしまった。
- Rule: `gwt-spec` Issue の完了判定は必ず Issue 本文の `Tasks` / 受け入れ基準 / PR本文 / 作業ツリー状態を同期させた上で行う。1つでも未同期なら「完了」と言わない。
- Checkpoint: 1. Issue 本文の `Tasks` を更新 2. 検証結果を Issue/PR に反映 3. `git status --short` を確認 4. ignore すべきローカル生成物が残っていれば `.gitignore` か cleanup を先に行う

### 2026-05-11 (SPEC #185 振り返り)

- Context: SPEC #185 (UnityCsReference 参照キャッシュ Phase 1) を実装した PR #186 が、CI で `Rust Format & Lint` 失敗を 3 回繰り返した。原因はローカル clippy が 0.1.94、CI clippy が 1.95.0 で、新規 lint (`collapsible_match`、`unnecessary_sort_by`、`await_holding_lock`) と `unused_variables` を見逃していた。
- Mistake: ローカルでの `cargo clippy --all-targets -- -D warnings` が clean だったため push 前検証を完了と見なした。
- Rule: 大きな PR を出す前に、CI と同じ rust toolchain で clippy を回す。`rustup show active-toolchain` を CI の `dtolnay/rust-toolchain@stable` 指定と突き合わせ、ずれていたら `rustup update stable` してから再検証。最低限 `cargo +stable clippy --all-targets -- -D warnings` を必ず通す。
- Checkpoint: 1. `rustup show active-toolchain` を実行 2. CI の rust-toolchain 指定と一致するか確認 3. 不一致なら `rustup update` 4. clippy 再実行

### 2026-05-11 (coverage gate)

- Context: SPEC #185 で新規 module `src/reference/*` を ~1,000 行追加した結果、Phase 1 第 1 commit 時点の line coverage が 89.15% まで下がり、CI `Rust Coverage >= 90% (required)` が 2 連続で fail した。
- Mistake: 新規 module の test を「主要パスを cover していれば十分」と簡略化したため、dispatcher / wiring / error paths が未 cover で全体閾値を割った。
- Rule: 大規模な新規 module を追加する PR では、追加直後に `cargo llvm-cov --all-targets --summary-only -- --test-threads=1` をローカルで 1 回実行して 90% を確認する。reference / dispatcher / CLI 配線 / error path も TDD の中で test を書く。
- Checkpoint: 1. 新規 module 着手時に test stub を先に書く (RED) 2. 実装ごとに対応する test を増やす 3. PR push 前にローカル llvm-cov で全体 90% を確認 4. 90% 未達なら不足 module を識別し、small test を集中投入

### 2026-05-11 (SPEC section markers)

- Context: SPEC #185 を `gwtd issue spec create --title ... -f <body>` で plain markdown 本文として作成したため、`<!-- sections: -->` コメントが付かず、`gwtd issue spec 185 --section spec` / `--edit tasks -f <file>` などの section 操作が `section 'spec' not found` で失敗した。
- Mistake: section 構造が必要であることを起票時に意識せず、`gwt-build-spec` の completion gate で tasks セクション更新ができなくなった。
- Rule: SPEC を起票するときは body 内に `## Spec` / `## Plan` / `## Tasks` / `## TDD` の見出しを揃え、必要なら `gwtd issue spec --edit <section>` を初回投入時から使う。`--edit` 経由なら gwtd 側が `<!-- sections: ... -->` コメントを管理してくれる。
- Checkpoint: 1. spec create 直後に `gwtd issue spec <n>` を実行して `<!-- sections: -->` が埋まっているか確認 2. 空ならその場で `--edit spec` / `--edit plan` / `--edit tasks` / `--edit tdd` で投入し直す
