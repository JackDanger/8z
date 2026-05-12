#!/usr/bin/env python3
"""
Compare benchmark results and flag regressions.

Supported input formats
-----------------------
Simple format (what bench.yml CI workflow produces):
    A single JSON file with the shape:
        {"benchmarks": [{"name": "...", "throughput_mb_s": 123.4}, ...]}
    Pass two such files as positional arguments:
        python3 scripts/bench_compare.py baseline.json current.json

Criterion directory format (local dev, criterion 0.5):
    A directory tree like target/criterion/ where each bench has:
        target/criterion/<bench>/new/estimates.json   (mean nanoseconds)
        target/criterion/<bench>/new/benchmark.json   (throughput info, optional)
    Pass two such directories as positional arguments:
        python3 scripts/bench_compare.py target/criterion.baseline/ target/criterion/

Exit codes
----------
    0  — all benchmarks within the 5 % regression threshold
    1  — one or more benchmarks regressed by more than 5 %
         (regressions are always printed; improvements are printed as well)
    2  — usage error or missing files
"""

import json
import os
import sys


THRESHOLD_PCT = 5.0   # regressions beyond this are failures


# ---------------------------------------------------------------------------
# Loaders
# ---------------------------------------------------------------------------

def _load_json(path: str) -> dict:
    try:
        with open(path) as fh:
            return json.load(fh)
    except Exception as exc:
        print(f"error: cannot load {path}: {exc}", file=sys.stderr)
        sys.exit(2)


def _load_simple(path: str) -> dict[str, float]:
    """Load from our simple {"benchmarks": [...]} format.

    Returns mapping of bench name -> throughput_mb_s.
    """
    data = _load_json(path)
    benches: dict[str, float] = {}
    for entry in data.get("benchmarks", []):
        name = entry.get("name")
        tput = entry.get("throughput_mb_s")
        if name is not None and tput is not None:
            benches[str(name)] = float(tput)
    return benches


def _load_criterion_dir(path: str) -> dict[str, float]:
    """Walk a criterion output directory tree and extract mean throughput.

    Criterion 0.5 stores per-benchmark results under:
        <path>/<bench-name>/new/estimates.json
    The 'mean' estimate is in nanoseconds per iteration.  We convert to an
    *inverse throughput* (ns/B is not directly available, so we just keep
    the mean-ns value and compare ratios — the direction is inverted vs
    MB/s: higher mean-ns = slower).

    We store the value as negative-throughput (1 / mean_ns) so that the
    comparison logic (higher = faster) stays consistent with the simple
    format.
    """
    benches: dict[str, float] = {}
    for bench_name in sorted(os.listdir(path)):
        estimates_path = os.path.join(path, bench_name, "new", "estimates.json")
        if not os.path.isfile(estimates_path):
            continue
        data = _load_json(estimates_path)
        mean_ns = data.get("mean", {}).get("point_estimate")
        if mean_ns is not None and float(mean_ns) > 0:
            # Store as throughput-proxy: 1e9 / mean_ns  (higher = faster)
            benches[bench_name] = 1e9 / float(mean_ns)
    return benches


def load_benches(path: str) -> dict[str, float]:
    """Auto-detect format from path and return name -> throughput mapping."""
    if os.path.isdir(path):
        return _load_criterion_dir(path)
    return _load_simple(path)


# ---------------------------------------------------------------------------
# Comparison logic
# ---------------------------------------------------------------------------

def compare(baseline_path: str, current_path: str) -> int:
    """Compare benchmark results; print deltas and return exit code."""
    baseline = load_benches(baseline_path)
    current = load_benches(current_path)

    if not baseline:
        print(f"error: no benchmarks found in baseline {baseline_path!r}", file=sys.stderr)
        return 2
    if not current:
        print(f"error: no benchmarks found in current {current_path!r}", file=sys.stderr)
        return 2

    # Find common benchmarks
    common = sorted(set(baseline) & set(current))
    if not common:
        print("error: no common benchmark names between baseline and current", file=sys.stderr)
        return 2

    regressions = []
    improvements = []
    neutral = []

    for name in common:
        base_val = baseline[name]
        cur_val = current[name]
        if base_val == 0:
            continue
        pct = (cur_val - base_val) / base_val * 100.0
        if pct < -THRESHOLD_PCT:
            regressions.append((name, pct))
        elif pct > THRESHOLD_PCT:
            improvements.append((name, pct))
        else:
            neutral.append((name, pct))

    # Print results
    all_results = (
        [(n, p, "REGRESS") for n, p in regressions]
        + [(n, p, "ok     ") for n, p in neutral]
        + [(n, p, "IMPROVE") for n, p in improvements]
    )
    for name, pct, tag in sorted(all_results, key=lambda x: x[1]):
        sign = "+" if pct >= 0 else ""
        print(f"  [{tag}] {name}: {sign}{pct:.1f}%")

    if regressions:
        print(f"\nFAIL: {len(regressions)} regression(s) exceed {THRESHOLD_PCT:.0f}% threshold.")
        return 1

    print(f"\nPASS: all {len(common)} benchmark(s) within {THRESHOLD_PCT:.0f}% threshold.")
    return 0


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> None:
    if len(sys.argv) < 3:
        print(
            "usage: bench_compare.py <baseline> <current>\n"
            "       <baseline> and <current> may each be a .json file (simple format)\n"
            "       or a criterion output directory.",
            file=sys.stderr,
        )
        sys.exit(2)
    sys.exit(compare(sys.argv[1], sys.argv[2]))


if __name__ == "__main__":
    main()
