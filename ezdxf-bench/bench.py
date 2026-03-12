"""
ezdxf Benchmark Runner
Called from the Rust harness with:
  python ezdxf-bench/bench.py --dir <bench_output/scale> --iterations <N>
Outputs JSON timing results to stdout.

ezdxf supports ASCII and Binary DXF read/write (no DWG).
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


def read_binary(path: str):
    """Read a binary DXF file with ezdxf.readfile (supports binary natively)."""
    return ezdxf.readfile(path)


def upgrade_doc(doc):
    """Copy entities to a new R2018 document so it can be saved."""
    new_doc = ezdxf.new(dxfversion="R2018")
    msp_src = doc.modelspace()
    msp_dst = new_doc.modelspace()
    for entity in msp_src:
        try:
            msp_dst.add_entity(entity.copy())
        except Exception:
            pass
    return new_doc


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
    # BINARY DXF PARSE benchmarks (DXB from disk)
    # -------------------------------------------------------------------
    binary_parse_results = []
    dxb_files = [("binary_mixed", "binary_mixed.dxb"), ("binary_lines", "binary_lines.dxb")]
    for label, filename in dxb_files:
        path = os.path.join(bench_dir, filename)
        if not os.path.exists(path):
            continue
        try:
            # Warmup
            read_binary(path)
            start = time.perf_counter()
            for _ in range(iterations):
                read_binary(path)
            elapsed = time.perf_counter() - start
            ms = (elapsed / iterations) * 1000.0
            binary_parse_results.append({"Label": label, "Ms": ms})
        except Exception as ex:
            print(f"Warning: ezdxf binary parse {label} failed: {ex}", file=sys.stderr)
    results["binary_parse"] = binary_parse_results

    # -------------------------------------------------------------------
    # BINARY DXF WRITE benchmarks (DXB to disk)
    # -------------------------------------------------------------------
    binary_write_results = []
    for label, src_name in [("binary_lines", "lines_only.dxf"), ("binary_mixed", "mixed.dxf")]:
        src_path = os.path.join(bench_dir, src_name)
        if not os.path.exists(src_path):
            continue
        try:
            doc = safe_read(src_path)
            out_path = os.path.join(bench_dir, f"write_{label}_ezdxf.dxb")
            # Warmup
            doc.saveas(out_path, fmt="bin")
            start = time.perf_counter()
            for _ in range(iterations):
                doc.saveas(out_path, fmt="bin")
            elapsed = time.perf_counter() - start
            ms = (elapsed / iterations) * 1000.0
            binary_write_results.append({"Label": label, "Ms": ms})
        except Exception as ex:
            print(f"Warning: ezdxf binary write {label} failed: {ex}", file=sys.stderr)
    results["binary_write"] = binary_write_results

    # -------------------------------------------------------------------
    # BINARY DXF ROUNDTRIP benchmarks (read DXB, write DXB)
    # -------------------------------------------------------------------
    binary_rt_results = []
    binary_mixed_src = os.path.join(bench_dir, "binary_mixed.dxb")
    if os.path.exists(binary_mixed_src):
        rt_path = os.path.join(rt_dir, "rt_binary_mixed_ezdxf.dxb")
        try:
            # Warmup
            doc = read_binary(binary_mixed_src)
            upgraded = upgrade_doc(doc)
            upgraded.saveas(rt_path, fmt="bin")
            start = time.perf_counter()
            for _ in range(iterations):
                doc = read_binary(binary_mixed_src)
                upgraded = upgrade_doc(doc)
                upgraded.saveas(rt_path, fmt="bin")
            elapsed = time.perf_counter() - start
            ms = (elapsed / iterations) * 1000.0
            binary_rt_results.append({"Label": "binary_mixed_roundtrip", "Ms": ms})
        except Exception as ex:
            print(f"Warning: ezdxf binary roundtrip failed: {ex}", file=sys.stderr)
    results["binary_roundtrip"] = binary_rt_results

    # -------------------------------------------------------------------
    # Output JSON (single line to stdout)
    # -------------------------------------------------------------------
    print(json.dumps(results))


if __name__ == "__main__":
    main()
