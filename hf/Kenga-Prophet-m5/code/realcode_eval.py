"""tools/realcode_eval.py — next-token accuracy on REAL Kenga code,
methodology identical to train_m3.py held-out (all positions per window,
non-overlapping windows, chunked forwards). Apples-to-apples model compare.
"""
import os
import sys
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat
import train_m3


def eval_model(model_path, codec, tok, K=128, chunk=128):
    info, tensors = kenchat.load_tensors(model_path)
    # build the training-side model so forward returns (B, K, V) logits
    m = train_m3.M3(info['vocab'], K, info['d'], info['h'],
                    info.get('layers', 1), np.random.RandomState(0))
    m.bout = tensors['bout']
    pm = m.params_map()
    for name in pm:
        pm[name][...] = tensors[name]
    files = []
    for root in ('kenga', 'examples'):
        for r, ds, fs in os.walk(root):
            for f in fs:
                if not f.endswith('.kenga'):
                    continue
                if ('kenga_seed_' in f or f.startswith('mid_prophet')
                        or f.startswith('pico_birth')):
                    continue
                files.append(os.path.join(r, f))
    import random
    random.seed(7)
    random.shuffle(files)
    tc = tt = 0
    for p in files[:12]:
        src = open(p, encoding='utf-8', errors='replace').read()
        t = tok(src)
        if len(t) < K + 1:
            continue
        arr_h = np.array(t, dtype=np.int32)
        idx_all = np.arange(K, len(arr_h), K)
        for s0 in range(0, len(idx_all), chunk):
            ch = idx_all[s0:s0 + chunk]
            wins = np.stack([arr_h[ch - K + j] for j in range(K)], axis=1)
            logits, _ = m.forward(wins)
            preds = logits.argmax(axis=-1)
            targets = np.stack([arr_h[ch - K + 1 + j] for j in range(K)], axis=1)
            tc += int((preds == targets).sum())
            tt += int(preds.size)
    return tc, tt


def main():
    os.environ.setdefault('M3_CODEC', '1')
    os.environ.setdefault('M3_CODEC_FILE', 'minds/kenga_full.pkl')
    codec = kenchat.load_codec_vocab('minds/kenga_full.pkl')
    tok = train_m3.make_codec_tokenize(train_m3.make_codec())
    for tag in sys.argv[1:] or ['m5', 'm42']:
        path = f'minds/mid_prophet_{tag}_w.txt'
        if not os.path.exists(path):
            print(f'{tag}: weights not found, skip')
            continue
        tc, tt = eval_model(path, codec, tok)
        print(f'{tag}: REAL-CODE next-token {tc}/{tt} = {100*tc/max(1,tt):.2f}%')


if __name__ == '__main__':
    main()
