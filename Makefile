.PHONY: quick route-check bench ship oracle-check status clean corpora corpora-clean help

## quick: run fast test suite (default)
quick:
	@cargo test --workspace

## route-check: placeholder for routing verification (future)
route-check:
	@echo "TODO: implement routing checks in future phases"

## bench: run criterion benchmarks
bench:
	@cargo bench --workspace

## ship: run full validation before release (local checks only)
ship:
	@echo "── fmt + clippy ──"
	@cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
	@echo "── tests ──"
	@cargo test --workspace --release
	@echo "── oracle ──"
	@$(MAKE) oracle-check
	@echo "✓ ship ready"

## oracle-check: run oracle tests (7zz round-trip)
oracle-check:
	@cargo test --workspace -- --include-ignored oracle

## status: print current focus and next task
status:
	@bash scripts/status.sh

## corpora: download all benchmark corpora to /tmp/7zippy-corpora/
corpora:
	@$(MAKE) -C corpora all

## corpora-clean: remove downloaded benchmark corpora from /tmp/7zippy-corpora/
corpora-clean:
	@$(MAKE) -C corpora clean

## clean: remove build artifacts
clean:
	@cargo clean

## help: show available targets
help:
	@grep -E "^##" Makefile | sed 's/^## /  /'
