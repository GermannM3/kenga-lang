#!/usr/bin/env bash
cd "$(dirname "$0")/.." || exit 1
export PYTHONIOENCODING=utf-8 OPENBLAS_NUM_THREADS=2
env M3_SEED=11 M3_TAG=m6sA M3_K=128 M3_D=128 M3_H=8 M3_L=6 M3_BATCH=32 M3_STEPS=600 \
    M3_EVAL_EVERY=300 M3_LR=0.002 M3_CLIP=1.0 M3_KEEP_COMMENTS=1 \
    M3_CODEC=1 M3_CODEC_FILE=minds/kenga_full.pkl M3_ONLY_FACTORY=1 \
    M3_FACTORY=minds/corpus_factory/split_v3/train.jsonl \
    python -u tools/train_m3.py > train_m6sA.log 2>&1
env M3_SEED=99 M3_TAG=m6sB M3_K=128 M3_D=128 M3_H=8 M3_L=6 M3_BATCH=32 M3_STEPS=600 \
    M3_EVAL_EVERY=300 M3_LR=0.002 M3_CLIP=1.0 M3_KEEP_COMMENTS=1 \
    M3_CODEC=1 M3_CODEC_FILE=minds/kenga_full.pkl M3_ONLY_FACTORY=1 \
    M3_FACTORY=minds/corpus_factory/split_v3/train.jsonl \
    python -u tools/train_m3.py > train_m6sB.log 2>&1
python - <<'PY'
import sys, os
sys.path.insert(0,'tools')
import kenchat, numpy as np
from zlineage import spectral_signature, pair_drift
a = kenchat.load_tensors('minds/mid_prophet_m6sA_w.txt')[1]
b = kenchat.load_tensors('minds/mid_prophet_m6sB_w.txt')[1]
L, det = pair_drift(spectral_signature(a), spectral_signature(b))
res = {'seed_pair_L': round(L,4), 'detail_mean_theta': round(float(np.mean([d['mean_theta_deg'] for d in det.values()])),2)}
json.dump(res, open('minds/seed_control_result.json','w'), indent=1)
print('SEED-CONTROL L =', res['seed_pair_L'])
PY
echo "[seedcontrol] done $(date +%H:%M:%S)"
