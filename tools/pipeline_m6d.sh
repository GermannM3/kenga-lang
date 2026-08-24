#!/usr/bin/env bash
# tools/pipeline_m6d.sh — lane 2: nl -> realgen -> report-build
set -u
cd "$(dirname "$0")/.." || exit 1
export PYTHONIOENCODING=utf-8 OPENBLAS_NUM_THREADS=2

python -u tools/nl_eval.py --model m6 --test minds/corpus_factory/split_v3/test.jsonl \
    -k 3 --limit 80 > minds/corpus_factory/eval_m6_nl.log 2>&1
echo "[lane2] nl done $(date +%H:%M:%S)"
python -u tools/realgen_eval.py --model m6 -k 4 > minds/corpus_factory/eval_m6_realgen.log 2>&1
echo "[lane2] realgen done $(date +%H:%M:%S)"

python -u tools/build_m6_report.py > /dev/null 2>&1 && echo "[lane2] report rebuilt" || true
