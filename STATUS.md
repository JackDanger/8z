# 8z STATUS

**Current focus:** Phase C closing: final fmt/clippy/test/oracle pass + push (task #10)

| Coder | Crate | Decode | Encode | Streaming | Oracle | Bench | Fuzz |
|---|---|---|---|---|---|---|---|
| Container header | 8z | ✅ | ✅ | n/a | ⬜ | ⬜ | ⬜ |
| Copy | 8z (in-tree) | ✅ | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| LZMA | lazippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| LZMA2 | lazippier | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| PPMd | pippyzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BZip2 | bzippy2 | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| Deflate | gzippy | ⬜ (blocked on gzippy lib PR) | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| Deflate64 | gzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (x86) | jumpzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (ARM) | jumpzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (ARM-Thumb) | jumpzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (PPC) | jumpzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (IA64) | jumpzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ (SPARC) | jumpzippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| BCJ2 | jumpzippier | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| Delta | deltazippy | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| AES + SHA-256 | lockzippy | ⬜ | ⬜ | n/a | ⬜ | ⬜ | ⬜ |

**Symbols**: ⬜ not started, 🟡 in progress, ✅ done, ❌ blocked

**Supporting machinery**:
- `make status` → runs `scripts/status.sh` which extracts the "Current focus" line and the next ⬜ row.
- Pre-commit hook warns if a PR touches `src/` but not `STATUS.md`.
- Each sibling crate ships its own internal `STATUS.md` for piece-level state (range coder, literal decoder, etc.) — the umbrella table is codec-level only so it stays scannable.
