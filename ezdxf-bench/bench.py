"""
ezdxf Benchmark Runner
Called from the Rust harness with:
  python ezdxf-bench/bench.py --dir <bench_output/scale> --iterations <N>
Outputs JSON timing results to stdout.

ezdxf supports ASCII DXF read/write only (no binary DXF, no DWG).
"""

import argparse
import json
import os
import sys
import time

import ezdxf
from ezdxf import recover
import math


def safe_read(path: str):
    """Read a DXF file using recover mode for maximum compatibility."""
    doc, _ = recover.readfile(path)
    return doc


def time_parse_dxf(path: str, iterations: int) -> float:
    """Time parsing a DXF file, return average ms."""
    # Warmup
    try:
        safe_read(path)
    except Exception:
        return None

    start = time.perf_counter()
    for _ in range(iterations):
        try:
            safe_read(path)
        except Exception:
            return None
    elapsed = time.perf_counter() - start
    return (elapsed / iterations) * 1000.0


def time_write_dxf(doc, path: str, iterations: int) -> float:
    """Time writing a DXF document, return average ms."""
    start = time.perf_counter()
    for _ in range(iterations):
        doc.saveas(path)
    elapsed = time.perf_counter() - start
    return (elapsed / iterations) * 1000.0


def time_roundtrip_dxf(src: str, dst: str, iterations: int) -> float:
    """Time read + write roundtrip, return average ms."""
    # Warmup
    try:
        doc = safe_read(src)
        doc.saveas(dst)
    except Exception:
        return None

    start = time.perf_counter()
    for _ in range(iterations):
        try:
            doc = safe_read(src)
            doc.saveas(dst)
        except Exception:
            return None
    elapsed = time.perf_counter() - start
    return (elapsed / iterations) * 1000.0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dir", required=True)
    parser.add_argument("--iterations", type=int, default=5)
    args = parser.parse_args()

    bench_dir = args.dir
    iterations = args.iterations

    if not os.path.isdir(bench_dir):
        print(f"Error: directory '{bench_dir}' does not exist.", file=sys.stderr)
        sys.exit(1)

    rt_dir = os.path.join(bench_dir, "roundtrip")
    os.makedirs(rt_dir, exist_ok=True)

    results = {}

    # -------------------------------------------------------------------
    # PARSE benchmarks (DXF from disk)
    # -------------------------------------------------------------------
    parse_results = []
    dxf_files = [
        "lines_only.dxf",
        "circles_only.dxf",
        "arcs_only.dxf",
        "ellipses_only.dxf",
        "mixed.dxf",
        "polylines.dxf",
        "3d_entities.dxf",
    ]
    for filename in dxf_files:
        path = os.path.join(bench_dir, filename)
        if not os.path.exists(path):
            continue
        label = os.path.splitext(filename)[0]
        ms = time_parse_dxf(path, iterations)
        if ms is not None:
            parse_results.append({"Label": label, "Ms": ms})
    results["parse"] = parse_results

    # -------------------------------------------------------------------
    # WRITE benchmarks (DXF to disk)
    # -------------------------------------------------------------------
    write_results = []

    lines_src = os.path.join(bench_dir, "lines_only.dxf")
    if os.path.exists(lines_src):
        try:
            doc = safe_read(lines_src)
            out_path = os.path.join(bench_dir, "write_lines_ezdxf.dxf")
            ms = time_write_dxf(doc, out_path, iterations)
            write_results.append({"Label": "lines_only", "Ms": ms})
        except Exception as ex:
            print(f"Warning: ezdxf write lines failed: {ex}", file=sys.stderr)

    mixed_src = os.path.join(bench_dir, "mixed.dxf")
    if os.path.exists(mixed_src):
        try:
            doc = safe_read(mixed_src)
            out_path = os.path.join(bench_dir, "write_mixed_ezdxf.dxf")
            ms = time_write_dxf(doc, out_path, iterations)
            write_results.append({"Label": "mixed", "Ms": ms})
        except Exception as ex:
            print(f"Warning: ezdxf write mixed failed: {ex}", file=sys.stderr)
    results["write"] = write_results

    # -------------------------------------------------------------------
    # ROUNDTRIP benchmarks (read from disk, write to disk)
    # -------------------------------------------------------------------
    rt_results = []
    if os.path.exists(mixed_src):
        rt_path = os.path.join(rt_dir, "rt_mixed_ezdxf.dxf")
        ms = time_roundtrip_dxf(mixed_src, rt_path, iterations)
        if ms is not None:
            rt_results.append({"Label": "mixed_roundtrip", "Ms": ms})
    results["roundtrip"] = rt_results

    # -------------------------------------------------------------------
    # Output JSON (single line to stdout)
    # -------------------------------------------------------------------
    print(json.dumps(results))


if __name__ == "__main__":
    main()
