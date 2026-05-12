# 7zippy STATUS

**Current focus:** Wave B work in progress. Deflate now routes through gzippy 0.6 (Phase-2-quality backend). BCJ/Delta sub-crate extraction and AES encrypt path in progress.

| Coder | Crate | Decode | Encode | Streaming | Oracle | Bench | Fuzz |
|---|---|---|---|---|---|---|---|
| Container header | 7zippy | ✅ | ✅ | n/a | ✅ | ✅ | ⬜ |
| Copy | 7zippy (in-tree) | ✅ | ✅ | ✅ | ✅ | ✅ | ⬜ |
| LZMA | lazippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| LZMA2 | lazippier | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| PPMd | pippyzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BZip2 | bzippy2 | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| Deflate | gzippy 0.6 (Phase 2 quality) | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| Deflate64 | deflate64 crate (in-tree) | ✅ | ⬜ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (x86) | 7zippy (lzma-rust2) | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (ARM) | 7zippy (lzma-rust2) | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (ARM-Thumb) | 7zippy (lzma-rust2) | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (PPC) | 7zippy (lzma-rust2) | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (IA64) | 7zippy (lzma-rust2) | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (SPARC) | 7zippy (lzma-rust2) | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ2 | jumpzippier | ✅ | ⬜ | ⬜ | ✅ | ⬜ | ⬜ |
| Delta | 7zippy (in-tree) | ✅ | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| AES + SHA-256 | lockzippy | ✅ | ⬜ | n/a | ✅ | ⬜ | ⬜ |

**Symbols**: ⬜ not started, 🟡 in progress, ✅ done, ❌ blocked

**Supporting machinery**:
- `make status` → runs `scripts/status.sh` which extracts the "Current focus" line and the next ⬜ row.
- Pre-commit hook warns if a PR touches `src/` but not `STATUS.md`.
- Each sibling crate ships its own internal `STATUS.md` for piece-level state (range coder, literal decoder, etc.) — the umbrella table is codec-level only so it stays scannable.
