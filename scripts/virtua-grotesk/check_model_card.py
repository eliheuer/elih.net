#!/usr/bin/env python3
"""Fact-check the Virtua Grotesk post's model card against a training run.

Every number in the post's section 06 model card and section 07 benchmark
table must come from an artifact on disk in the run directory. This script
derives each fact from the artifacts and reports whether the post states it.

Usage (from the elih.net repo root):

    python3 scripts/virtua-grotesk/check_model_card.py            # newest run
    python3 scripts/virtua-grotesk/check_model_card.py --run v08  # specific run

Artifacts read, per fact:
  config.json          architecture (layers/dims/heads), context, vocabulary
  weights.safetensors  parameter count, dtype, checkpoint size (decimal MB)
  run.log              corpus counts (OFL pairs, graded pairs, train glyphs),
                       session wall clock
  bench.json           section 07 table (must be multi-seed to be citable)

Exit code 1 if any check fails, so it can run in CI or a pre-publish hook.
"""

import argparse
import json
import re
import struct
import sys
from pathlib import Path

POST = Path(__file__).resolve().parents[2] / "src/content/blog/virtua-grotesk/index.mdx"
RUNS = Path.home() / "GH/repos/font-garden-lab/runs"


def read_safetensors(path):
    with open(path, "rb") as f:
        n = struct.unpack("<Q", f.read(8))[0]
        header = json.loads(f.read(n))
    params = 0
    dtypes = set()
    for key, val in header.items():
        if key == "__metadata__":
            continue
        count = 1
        for dim in val["shape"]:
            count *= dim
        params += count
        dtypes.add(val["dtype"])
    return params, dtypes, path.stat().st_size


def parse_log(path):
    text = path.read_text()
    facts = {}
    m = re.search(r"OFL corpus: (\d+) train / (\d+) val", text)
    if m:
        facts["ofl_train"] = int(m.group(1))
    m = re.search(r"green-approved bold pairs: (\d+)", text)
    if m:
        facts["graded_pairs"] = int(m.group(1))
    m = re.search(r"train glyphs (\d+)", text)
    if m:
        facts["train_glyphs"] = int(m.group(1))
    times = re.findall(r"=== .* (?:started|finished): \w+ (\w+ +\d+ [\d:]+) \w+ (\d+)", text)
    if len(times) >= 2:
        from datetime import datetime

        parse = lambda t: datetime.strptime(f"{t[0]} {t[1]}", "%b %d %H:%M:%S %Y")
        delta = parse(times[-1]) - parse(times[0])
        h, rem = divmod(int(delta.total_seconds()), 3600)
        facts["wall_clock"] = f"{h}h{rem // 60:02d}m"
    return facts


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", help="run name under runs/, e.g. v08 (default: newest)")
    ap.add_argument("--post", type=Path, default=POST)
    args = ap.parse_args()

    if args.run:
        run = RUNS / args.run
    else:
        candidates = sorted(d for d in RUNS.glob("v*") if (d / "config.json").exists())
        if not candidates:
            sys.exit(f"no runs with config.json under {RUNS}")
        run = candidates[-1]
    post = args.post.read_text()
    print(f"run:  {run}")
    print(f"post: {args.post}\n")

    config = json.loads((run / "config.json").read_text())
    params, dtypes, ckpt_bytes = read_safetensors(run / "weights.safetensors")
    log = parse_log(run / "run.log")

    # (label, expected string that must appear in the post, source)
    expected = [
        ("architecture", f"{config['layers']} layers, {config['dims']} dims, {config['heads']} heads", "config.json"),
        ("parameters", f"{params / 1e6:.2f}M", "weights.safetensors"),
        ("context", f"{config['max_len']:,} tokens", "config.json"),
        ("vocabulary", f"{config['vocab_size']:,} tokens", "config.json"),
        ("checkpoint size", f"{round(ckpt_bytes / 1e6)} MB", "weights.safetensors (decimal MB, what HF displays)"),
        ("checkpoint dtype", {"F32": "fp32", "F16": "fp16", "BF16": "bf16"}.get(next(iter(dtypes)), "?"), "weights.safetensors"),
    ]
    if "ofl_train" in log:
        expected.append(("pretraining pairs", f"{log['ofl_train']:,}", "run.log OFL corpus line"))
    if "graded_pairs" in log:
        expected.append(("fine-tuning pairs", f"{log['graded_pairs']} human-graded", "run.log green-approved line"))
    if "train_glyphs" in log:
        expected.append(("corpus glyphs", f"{log['train_glyphs']} glyphs", "run.log train glyphs"))
    if "wall_clock" in log:
        expected.append(("session wall clock", log["wall_clock"], "run.log start/finish stamps"))

    failures = 0
    for label, needle, source in expected:
        ok = needle in post
        print(f"{'ok  ' if ok else 'FAIL'} {label}: {needle!r}  [{source}]")
        failures += not ok

    bench_path = run / "bench.json"
    if bench_path.exists():
        bench = json.loads(bench_path.read_text())
        seeds = bench.get("seeds", 1)
        if seeds < 2:
            print(f"FAIL bench.json has seeds={seeds}; the published table needs the "
                  "multi-seed artifact (rerun eval_bench --seeds 5)")
            failures += 1
        else:
            base = bench["baseline"]
            model = bench["model"]
            rows = [
                ("baseline MAE", f"{base['mae']:.1f}"),
                ("baseline Chamfer", f"{base['chamfer']:.1f}"),
                ("baseline IoU", f"{base['iou']:.3f}"),
                ("model MAE", f"{model['mae']['mean']:.1f} ± {model['mae']['std']:.1f}"),
                ("model Chamfer", f"{model['chamfer']['mean']:.1f} ± {model['chamfer']['std']:.1f}"),
                ("model IoU", f"{model['iou']['mean']:.3f} ± {model['iou']['std']:.3f}"),
            ]
            for label, needle in rows:
                ok = needle in post
                print(f"{'ok  ' if ok else 'FAIL'} {label}: {needle!r}  [bench.json, {seeds} seeds]")
                failures += not ok
    else:
        print("note: no bench.json in run dir; section 07 table unchecked")

    print(f"\n{failures} failure(s)" if failures else "\nall facts check out")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
