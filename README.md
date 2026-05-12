# 8z

8z is a pure-Rust 7z archive implementation. Umbrella over a family of codec sub-crates (`lazippy`, `bzippy2`, `gzippy`, …). Status: scaffolding; see [STATUS.md](STATUS.md).

## Build and test

```bash
cargo build
cargo test --workspace
make oracle-check
```

Consult the [implementation plan](/.claude/plans/use-github-com-jackdanger-gzippy-as-insp-starry-prism.md) for current scope and dispatch sequence.
