#!/usr/bin/env bash
cd "$(dirname "$0")/.." || exit 1
export PYTHONIOENCODING=utf-8 OPENBLAS_NUM_THREADS=2
python -u tools/repair_eval.py --model rp1 --codec minds/kenga_fix.pkl --marker FIX --limit 60 -k 4 > minds/corpus_factory/eval_rp1_repair_v3.log 2>&1
echo done >> minds/corpus_factory/eval_rp1_repair_v3.log
