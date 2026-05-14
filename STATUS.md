# 7zippy STATUS

**Current focus:** Drain the PR backlog (#8 multi-coder folder) with real review on each, then close Phase 1's remaining encode + oracle gaps (BCJ2 encode, Deflate64 encode via gzippy ≥0.6, BCJ-variant oracle rows). Phase 2 native rewrites begin after this gate.

**Phase 1 exit criterion** (user-set, 2026-05-13): every codec row reaches **Decode=✅, Encode=✅, Oracle=✅**. Streaming, Bench, and Fuzz columns are not part of the Phase 1 gate.

| Coder | Crate | Decode | Encode | Streaming | Oracle | Bench | Fuzz |
|---|---|---|---|---|---|---|---|
| Container header | 7zippy | ✅ | ✅ | n/a | ✅ | ✅ | ⬜ |
| Copy | 7zippy (in-tree) | ✅ | ✅ | ✅ | ✅ | ✅ | ⬜ |
| LZMA | lazippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| LZMA2 | lazippier | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| PPMd | pippyzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BZip2 | bzippy2 | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| Deflate | flate2 (Phase 1) | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| Deflate64 | deflate64 crate (in-tree) | ✅ | ⬜ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (x86) | jumpzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (ARM) | jumpzippy | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (ARM-Thumb) | jumpzippy | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (PPC) | jumpzippy | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (IA64) | jumpzippy | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (SPARC) | jumpzippy | ✅ | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ2 | jumpzippier | ✅ | ⬜ | ⬜ | ✅ | ⬜ | ⬜ |
| Delta | deltazippy | ✅ | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| AES + SHA-256 | lockzippy 0.0.2 | ✅ | 🟡 | n/a | ✅ | ⬜ | ⬜ |

**Symbols**: ⬜ not started, 🟡 in progress, ✅ done, ❌ blocked

**Supporting machinery**:
- `make status` → runs `scripts/status.sh` which extracts the "Current focus" line and the next ⬜ row.
- Pre-commit hook warns if a PR touches `src/` but not `STATUS.md`.
- Each sibling crate ships its own internal `STATUS.md` for piece-level state (range coder, literal decoder, etc.) — the umbrella table is codec-level only so it stays scannable.
