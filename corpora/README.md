# corpora — benchmark corpus fetcher

This directory contains a `Makefile` that downloads standard compression
benchmark corpora to `/tmp/7zippy-corpora/` on demand.  Nothing downloaded here is
ever committed to the repository; the only committed files are:

- `Makefile` — the downloader itself
- `SHA256SUMS` — canonical SHA-256 hashes for integrity verification
- `fixtures/` — tiny hand-crafted archive fixtures from task #5 (committed)
- `baselines/` — criterion baseline JSON snapshots (committed)

## Quick start

```sh
# Fetch a single corpus
make -C corpora silesia     # ~76 MB download -> /tmp/7zippy-corpora/silesia/
make -C corpora enwik8      # ~36 MB download -> /tmp/7zippy-corpora/enwik8/
make -C corpora calgary     # ~3 MB download  -> /tmp/7zippy-corpora/calgary/

# Fetch everything (~330 MB total)
make -C corpora all

# Or from the repo root
make corpora
```

Each target is idempotent: re-running it when the archive already exists on
disk does nothing.

## Using corpora in benchmarks

Benchmarks under `benches/` load fixtures via `common::load_fixture(name)` for
tiny committed files.  For the large downloaded corpora, use
`common::load_corpus(name)` (to be wired up in a future task):

```rust
// Looks up /tmp/7zippy-corpora/<name>/<name> (or similar layout)
let data = common::load_corpus("enwik8");
```

See `benches/common/mod.rs` for the current helper implementations.

## Verifying integrity

```sh
make -C corpora verify    # re-check SHA-256 of all downloaded archives
make -C corpora list      # print paths and disk sizes
```

## Cleaning up

```sh
make -C corpora clean     # rm -rf /tmp/7zippy-corpora/
```

## Upstream sources

| Corpus | URL | Size |
|---|---|---|
| Silesia | http://mattmahoney.net/dc/silesia.zip | ~76 MB (zip), ~270 MB uncompressed |
| enwik8 | http://mattmahoney.net/dc/enwik8.zip | ~36 MB (zip), 100 MB uncompressed |
| Calgary | http://mattmahoney.net/dc/calgary.tar | ~3 MB |

The Silesia corpus was assembled by Przemyslaw Skibinski.  enwik8 is a
100-million-byte subset of the English Wikipedia prepared by Matt Mahoney.
The Calgary corpus was originally published by Ian H. Witten, Timothy C. Bell,
and John G. Clearly.
