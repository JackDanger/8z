# Iteration Loop — How 7zippy Gets Built

This document describes the repeating cycle that moves us from one task to the next.

## The Cycle

### 1. Read STATUS.md

Open [`STATUS.md`](../STATUS.md) at the repo root. The table shows which codec is "Current focus" and which rows are ⬜ (not started).

Example:
```
**Current focus:** lazippy / range coder / decoder
```

The next ⬜ row will be highlighted here.

### 2. Opus Picks the Task

The Opus agent (oversight-only) reads STATUS.md and decides which ⬜ row to tackle next. For example:
- "Container header parser is 🟡 (in flight). Next: finish it."
- "Copy coder is ⬜. Let's prove the pipeline works with the simplest algorithm."
- "LZMA range coder is ⬜. Begin the first codec."

### 3. Opus Dispatches an Agent

Opus crafts a self-contained prompt with:
- **What to build** — specific files, modules, tests
- **Acceptance criteria** — "these tests must pass", "this makes STATUS.md row ✅"
- **Guardrails** — "DO NOT touch src/ except for X.rs", "DO NOT push"

Example dispatch:
```
Task: Implement the Copy coder in 7zippy

Build src/pipeline/copy.rs with:
  - CopyCoder struct implementing the Coder trait
  - One test: decode 1MB random input via Copy, assert bytes equal input

Acceptance:
  - cargo test src/tests/layer3_per_coder.rs::copy_smoke — passes
  - cargo test src/tests/layer4_pipeline.rs::round_trip_copy_coder — passes
  - cargo test src/tests/layer5_cross.rs::copy_oracle_round_trip -- --include-ignored — passes

Then:
  - Update STATUS.md row "Copy" columns Decode/Encode to ✅
  - Note any deviations from the plan

Stop if: Cargo.toml is missing or needs editing (that's task #4).
```

Opus then spawns:
- **Haiku** for mechanical work (templates, renames, `Makefile` targets, script scaffolding)
- **Sonnet** for judgment-heavy work (byte-level parsing, trait design, error handling, tests)

### 4. Agent Reports Back

The agent runs `cargo test --workspace --release`, `make oracle-check`, etc., then summarizes:
- Files created (tree view)
- Test output (pass/fail)
- Any issues found
- Deviations from the plan

### 5. Opus Reviews and Updates STATUS.md

Opus:
1. Spot-reads the files the agent touched (especially tests)
2. Confirms acceptance criteria met
3. Updates the row in STATUS.md:
   - ⬜ → 🟡 if in progress
   - 🟡 → ✅ if done
   - ✅ → ❌ if work revealed it's blocked
4. Updates the "Current focus" line to the **next** ⬜ row

If something is broken:
- Opus sends a follow-up message to the agent (warm cache)
- Agent fixes and re-reports
- Repeat until green

### 6. Repeat

Loop back to step 1 with the updated STATUS.md. The next agent reads the new "Current focus" and starts.

## Tools and Milestones

### Verification Commands

Before finishing each task, run:
```bash
cargo fmt --check                           # formatting
cargo clippy --workspace --all-targets -- -D warnings  # lints
cargo test --workspace --release            # correctness
make oracle-check                           # 7zz round-trip
make status                                 # print current focus
```

### Milestone Checklist (end of each task)

- [ ] All tests pass (`cargo test --workspace --release`)
- [ ] Oracle tests pass (if codec work): `cargo test -- --include-ignored oracle`
- [ ] No clippy warnings
- [ ] STATUS.md row(s) updated with correct symbols
- [ ] "Current focus" line points to next ⬜
- [ ] `git status` shows only non-src files (unless this task touched src/)

### Cross-Repo Coordination

When a sub-crate (lazippy, bzippy2, etc.) is ready:

1. **Agent creates the repo** — `gh repo create JackDanger/lazippy --public`
2. **Agent pushes the template** — all files committed, CI green
3. **Opus wires it into 7zippy** — add dependency, dispatch next round
4. **Back to step 1** — STATUS.md now shows the new crate live

## Why This Works

- **Explicit position tracking** — STATUS.md is never out of date. Before each task, you know exactly what was done and what's next.
- **Decoupled agent work** — agents can run in parallel (Container + Copy coder + oracle harness at the same time, for example) because their file sets don't overlap.
- **Cheap verification** — oracle tests against `7zz` are fast and deterministic. No guessing if our output is correct.
- **Iteration speed** — Opus reviews in minutes, not hours. Agent cycle time is < 30 min for most slices.

## Current State

**See [STATUS.md](../STATUS.md) for the live position.**

Run `make status` to print it:
```bash
$ make status
Current focus:
  7zippy umbrella scaffolding (Phase C) / next: container parser

Next task:
  Container header
```

---

**Next steps**: read [CLAUDE.md](../CLAUDE.md) for development rules, or [ROADMAP.md](../ROADMAP.md) for high-level order of attack.
