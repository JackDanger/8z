# 7zippy STATUS

**Current focus:** Phase 1 closure COMPLETE. Every codec row has Decode=✅, Encode=✅, Oracle=✅. Phase 2 native rewrites begin: lazippy (LZMA range coder) is the first target — see `memory/7zippy-wrappers-then-native.md`.

**Phase 1 exit criterion** (user-set, 2026-05-13): every codec row reaches **Decode=✅, Encode=✅, Oracle=✅**. Streaming, Bench, and Fuzz columns are not part of the Phase 1 gate.

| Coder | Crate | Decode | Encode | Streaming | Oracle | Bench | Fuzz |
|---|---|---|---|---|---|---|---|
| Container header | 7zippy | ✅ | ✅ | n/a | ✅ | ✅ | ⬜ |
| Copy | 7zippy (in-tree) | ✅ | ✅ | ✅ | ✅ | ✅ | ⬜ |
| LZMA | lazippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| LZMA2 | lazippier | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| PPMd | pippyzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BZip2 | bzippy2 | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| Deflate | gzippy 0.8 | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| Deflate64 | gzippy 0.8 | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (x86) | jumpzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (ARM) | jumpzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (ARM-Thumb) | jumpzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (PPC) | jumpzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (IA64) | jumpzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ (SPARC) | jumpzippy | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| BCJ2 | jumpzippier 0.0.2 | ✅ | ✅ | ⬜ | ✅ | ⬜ | ⬜ |
| Delta | deltazippy | ✅ | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| AES + SHA-256 | lockzippy 0.0.2 | ✅ | ✅ | n/a | ✅ | ⬜ | ⬜ |

**Symbols**: ⬜ not started, 🟡 in progress, ✅ done, ❌ blocked

**Supporting machinery**:
- `make status` → runs `scripts/status.sh` which extracts the "Current focus" line and the next ⬜ row.
- Pre-commit hook warns if a PR touches `src/` but not `STATUS.md`.
- Each sibling crate ships its own internal `STATUS.md` for piece-level state (range coder, literal decoder, etc.) — the umbrella table is codec-level only so it stays scannable.
