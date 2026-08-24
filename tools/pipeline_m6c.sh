#!/usr/bin/env bash
# tools/pipeline_m6c.sh — lane 1: bind -> factory -> report-build (if others done)
set -u
cd "$(dirname "$0")/.." || exit 1
export PYTHONIOENCODING=utf-8 OPENBLAS_NUM_THREADS=2

python -u tools/corpus_eval.py --model m6 --test minds/corpus_factory/split_v3/test.jsonl \
    --category bind --limit 128 -k 4 > minds/corpus_factory/eval_m6_bind.log 2>&1
echo "[lane1] bind done $(date +%H:%M:%S)"
python -u tools/corpus_eval.py --model m6 --test minds/corpus_factory/split_v3/test.jsonl \
    --limit 60 -k 4 > minds/corpus_factory/eval_m6_factory.log 2>&1
echo "[lane1] factory done $(date +%H:%M:%S)"

# build report only when all four logs contain final tables
python -u tools/build_m6_report.py > /dev/null 2>&1 && echo "[lane1] report rebuilt" || true
