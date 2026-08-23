#!/usr/bin/env bash
# tools/pipeline_m6.sh — M6: K=512, Factory v3 (NL headers + str/list).
set -u
cd "$(dirname "$0")/.." || exit 1
export PYTHONIOENCODING=utf-8 OPENBLAS_NUM_THREADS=4

echo "[m6] train start $(date +%H:%M:%S)"
env M3_TAG=m6 M3_K=512 M3_D=128 M3_H=8 M3_L=6 M3_BATCH=16 M3_STEPS=2400 \
    M3_EVAL_EVERY=100 M3_LR=0.002 M3_CLIP=1.0 \
    M3_KEEP_COMMENTS=1 \
    M3_CODEC=1 M3_CODEC_FILE=minds/kenga_full.pkl \
    M3_REAL_SPLIT=0.1 M3_INCLUDE_BIG=1 \
    M3_FACTORY=minds/corpus_factory/split_v3/train.jsonl \
    M3_FACTORY_HOLDOUT=minds/corpus_factory/split_v3/test.jsonl \
    python -u tools/train_m3.py > train_m6.log 2>&1

echo "[m6] nl_eval $(date +%H:%M:%S)"
python -u tools/nl_eval.py --model m6 --test minds/corpus_factory/split_v3/test.jsonl \
    -k 4 --limit 200 > minds/corpus_factory/eval_m6_nl.log 2>&1

echo "[m6] bind eval $(date +%H:%M:%S)"
python -u tools/corpus_eval.py --model m6 --test minds/corpus_factory/split_v3/test.jsonl \
    --category bind --limit 128 -k 4 > minds/corpus_factory/eval_m6_bind.log 2>&1

echo "[m6] factory eval $(date +%H:%M:%S)"
python -u tools/corpus_eval.py --model m6 --test minds/corpus_factory/split_v3/test.jsonl \
    --limit 60 -k 4 > minds/corpus_factory/eval_m6_factory.log 2>&1

echo "[m6] realgen v2 $(date +%H:%M:%S)"
python -u tools/realgen_eval.py --model m6 -k 4 > minds/corpus_factory/eval_m6_realgen.log 2>&1

echo "[m6] done $(date +%H:%M:%S)"
