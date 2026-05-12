#!/usr/bin/env python3
"""
Compare two criterion benchmark JSON outputs.
Usage: python3 scripts/bench_compare.py [baseline.json] [current.json]
Exit code: 0 if all benches within 5% threshold, 1 otherwise.
"""

import json
import sys
import os

def load_json(path):
    """Load criterion JSON; return empty dict if file missing."""
    if not os.path.exists(path):
        print(f"Warning: {path} not found", file=sys.stderr)
        return {}
    try:
        with open(path) as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading {path}: {e}", file=sys.stderr)
        return {}

def extract_benches(data):
    """Extract benchmark times from criterion JSON."""
    benches = {}
    if "benchmarks" in data:
        for bench_name, bench_data in data["benchmarks"].items():
            if "estimated_value" in bench_data:
                benches[bench_name] = bench_data["estimated_value"]
    return benches

def compare(baseline_path, current_path):
    """Compare benchmarks; return (ok, report)."""
    baseline = extract_benches(load_json(baseline_path))
    current = extract_benches(load_json(current_path))

    if not baseline or not current:
        print("Insufficient data for comparison", file=sys.stderr)
        return True, ""

    threshold = 1.05  # 5% slower
    regressions = []

    for name in current:
        if name in baseline:
            ratio = current[name] / baseline[name]
            pct = (ratio - 1) * 100
            if ratio > threshold:
                regressions.append((name, pct))

    if regressions:
        report = "Benchmark regressions detected:\n"
        for name, pct in regressions:
            report += f"  {name}: +{pct:.1f}%\n"
        return False, report
    else:
        return True, "All benchmarks OK\n"

if __name__ == "__main__":
    baseline = sys.argv[1] if len(sys.argv) > 1 else "baseline.json"
    current = sys.argv[2] if len(sys.argv) > 2 else "current.json"

    ok, report = compare(baseline, current)
    print(report, end="")
    sys.exit(0 if ok else 1)
