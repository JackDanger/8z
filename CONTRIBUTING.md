# Contributing to 8z

## Adding a New Codec Sub-Crate

8z uses a "zippy family" naming convention: codec algorithms live in their own public repos and crates as dependencies (like gzippy for DEFLATE). The process is straightforward:

### 1. Clone the `lazippy` Template

`lazippy` is the canonical template. Every codec crate follows its shape:

```bash
git clone https://github.com/JackDanger/lazippy your-crate-name
cd your-crate-name
git checkout --orphan main  # new history
```

### 2. Rename and Swap

Update `Cargo.toml`, README, and source module names to match your algorithm:

```toml
[package]
name = "bzippy2"  # or pippyzippy, jumpzippy, etc.
description = "Pure-Rust BZip2 encoder/decoder, part of the 8z umbrella"
repository = "https://github.com/JackDanger/bzippy2"
```

Update `src/lib.rs` to export your codec module instead of LZMA stubs.

### 3. Hook into 8z's Dispatch

In `8z`'s `src/pipeline/dispatch.rs`, add a match arm for your method ID:

```rust
match method_id {
    MethodId::Copy => Box::new(copy::CopyCoder),
    MethodId::Lzma => Box::new(lazippy_wrapper::LzmaCoder),
    MethodId::Bzip2 => Box::new(bzippy2_wrapper::Bzip2Coder),  // Add here
    // ...
}
```

### 4. Update STATUS.md

Add a row to the umbrella `STATUS.md` table (or promote ⬜ to 🟡 if the row already exists):

```markdown
| BZip2 | bzippy2 | 🟡 | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
```

Initial status: decoder in flight (🟡), everything else ⬜.

### 5. Wire Feature Gates in 8z's Cargo.toml

In the umbrella's `Cargo.toml`:

```toml
[dependencies]
bzippy2 = { version = "0.0.1", path = "../bzippy2", optional = true }

[features]
bzip2 = ["dep:bzippy2"]
default = ["lzma", "lzma2", "ppmd", "bzip2", "deflate", "bcj", "bcj2", "delta", "crypto"]
```

The feature gate lets 8z compile even when sub-crate repos don't exist yet.

### 6. Pre-Commit / Pre-Push Checks

Your new crate inherits the gzippy-style git hooks (auto-installed by `build.rs`):

**`scripts/pre-commit`** — run before committing:
```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

**`scripts/pre-push`** — prevent direct pushes to `main`:
```bash
# Requires --no-verify to bypass
git push --no-verify origin my-branch
```

Override locally if needed (for debugging only), but commits must be clean.

### 7. Oracle Tests

Every new codec must ship with round-trip tests against `7zz`:

In `tests/vectors.rs`, add:
```rust
#[test]
fn test_bzip2_round_trip_via_oracle() {
    let input = b"the quick brown fox...";
    let spec = CoderSpec::bzip2(2);  // block size 2
    let archive = oracle::seven_zip_compress(input, &spec);
    let output = oracle::seven_zip_decompress(&archive);
    assert_eq!(input, &output[..]);
}
```

The oracle harness shells out to `7zz a -t7z -m0=04020201 …` (adjust method ID per algorithm).

### 8. Benchmarks

Add codec-level benchmarks to your crate's `benches/` directory using `criterion` with `Throughput::Bytes`:

```rust
fn benchmark_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    group.throughput(Throughput::Bytes(fixture.len() as u64));
    group.bench_function("fixture", |b| {
        b.iter(|| bzippy2::decode(&fixture))
    });
}
```

8z's umbrella benchmarks in `benches/archive_read.rs` will automatically run codec benchmarks for comparison.

## Code Review Expectations

- **Correctness first** — oracle tests pass, no unsafe without clear justification (`// SAFETY: …`).
- **One production path** — know which function the CLI calls; test that function.
- **Performance budgets** — micro-benches gate regressions. See `make bench` / `make ship`.
- **Zero dead code** — every module reachable from production. No experiments on `main`.
- **Documentation** — module-level doc comments explaining the algorithm shape.

See [CLAUDE.md](CLAUDE.md) for the full development guide and [docs/7z_FORMAT_NOTES.md](docs/7z_FORMAT_NOTES.md) for 7z method-ID pointers.
