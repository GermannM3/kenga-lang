#!/usr/bin/env bash
# tools/pipeline_m53_rp0.sh — resilient sequential pipeline.
# Stage 1: train M5.3          -> train_m53.log, marker: "step 2399" in log
# Stage 2: bind/factory/realgen evals for m53
# Stage 3: train rp0 (repair)  -> train_rp0.log
# Stage 4: repair_eval for rp0
set -u
cd "$(dirname "$0")/.." || exit 1
export PYTHONIOENCODING=utf-8 OPENBLAS_NUM_THREADS=4

echo "[pipeline] stage1: m53 train start $(date +%H:%M:%S)"
env M3_TAG=m53 M3_K=128 M3_D=128 M3_H=8 M3_L=6 M3_BATCH=64 M3_STEPS=2400 \
    M3_EVAL_EVERY=100 M3_LR=0.002 M3_CLIP=1.0 \
    M3_CODEC=1 M3_CODEC_FILE=minds/kenga_full.pkl \
    M3_REAL_SPLIT=0.1 M3_INCLUDE_BIG=1 \
    M3_FACTORY=minds/corpus_factory/split_v2/train.jsonl \
    M3_FACTORY_HOLDOUT=minds/corpus_factory/split_v2/test.jsonl \
    python -u tools/train_m3.py > train_m53.log 2>&1

echo "[pipeline] stage2: m53 evals $(date +%H:%M:%S)"
python -u tools/corpus_eval.py --model m53 --test minds/corpus_factory/split_v2/test.jsonl --category bind --limit 144 -k 4 > minds/corpus_factory/eval_m53_bind.log 2>&1
python -u tools/corpus_eval.py --model m53 --test minds/corpus_factory/split_v2/test.jsonl --limit 40 -k 4 > minds/corpus_factory/eval_m53_factory.log 2>&1
python -u tools/realgen_eval.py --model m53 -k 4 > minds/corpus_factory/eval_m53_realgen.log 2>&1

echo "[pipeline] stage3: rp0 repair train $(date +%H:%M:%S)"
env M3_TAG=rp0 M3_K=128 M3_D=128 M3_H=8 M3_L=6 M3_BATCH=64 M3_STEPS=2400 \
    M3_EVAL_EVERY=200 M3_LR=0.002 M3_CLIP=1.0 \
    M3_CODEC=1 M3_CODEC_FILE=minds/kenga_full.pkl \
    M3_ONLY_EXTRA=1 M3_EXTRA_DIR=minds/repair_corpus \
    python -u tools/train_m3.py > train_rp0.log 2>&1

echo "[pipeline] stage4: rp0 repair eval $(date +%H:%M:%S)"
python -u tools/repair_eval.py --model rp0 -k 4 > minds/corpus_factory/eval_rp0_repair.log 2>&1

echo "[pipeline] done $(date +%H:%M:%S)"
