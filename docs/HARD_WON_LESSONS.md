# Hard-Won Lessons

This file records discoveries and gotchas encountered during 7zippy development.

## Day 1: Oracle Testing is Non-Negotiable

**Lesson**: When in doubt, ask the 7zz CLI what it produces — it's the canonical reference.

The 7z format spec (7zFormat.txt) is dense and ambiguous. Rather than debate, shell out to `7zz a -t7z -m0=... <file>` and verify our output matches byte-for-byte. The oracle harness (`src/tests/oracle.rs`) makes this cheap:

```rust
#[test]
fn test_lzma_round_trip_via_oracle() {
    let input = b"...";
    let archive = oracle::seven_zip_compress(input, &MethodId::Lzma);
    let output = oracle::seven_zip_decompress(&archive);
    assert_eq!(input, &output[..]);
}
```

If our output differs from `7zz` on the same input, **we're wrong**. Full stop. Iterate until byte-perfect.

---

More lessons TBD as we build. This section grows with each agent round.
