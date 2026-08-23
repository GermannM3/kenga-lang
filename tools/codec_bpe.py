"""tools/codec_bpe.py — learned codec for Kenga.

Three codec variants (env-selectable):

  BPE_DIGITS=0  (default) -> 64-token codec (M3.2): 27 syntax + ID + NUM
                             + 26 lowercase letters + 10 merges.
  BPE_DIGITS=1             -> 73-token codec (M3.7): NUM replaced by 10 digit
                             tokens 0-9 (numbers are spellable digit-by-digit).
  BPE_FULL=1               -> full-alphabet codec (M4.x): 27 syntax + ID
                             + 10 digits + 26 lower + 26 UPPER + '_'
                             + merges. Spellable identifiers may contain
                             uppercase/underscore/digits, so real Kenga code
                             (Tensor, t_get, KVal, wf1, size_t) is representable.

Learned from the TRAINING corpus only (kenga_seed_* held out), same corpus
as train_m3.py.

Outputs (tag = 'bpe' | 'digits' | 'full'):
  minds/kenga_<tag>_vocab.txt   vocab (id -> token)
  minds/kenga_<tag>_merges.txt  learned merges (a b -> ab), reproducibility
  minds/kenga_<tag>.pkl         codec object (encode/decode helpers)
"""
import os
import re
import collections
import pickle

SKIP_BIG = {
    'bc_src_c.kenga','more.kenga','lower_kv.kenga','lower_c.kenga',
    'rt_prophet.kenga','native_ml.kenga','rt_vm.kenga','rt_tensor.kenga',
    'rt_kval_tape.kenga','rt_kval_mem.kenga',
}

KEYWORDS = {'fn','return','let','if','else','while','for','i64','println'}

# Original 28-token syntax set (minus ID; ID becomes the fallback below).
SYNTAX = [
    'fn',     'return', 'let',    'if',     'else',   'while',
    'for',    'i64',    ':',      ',',      ';',      '{',
    '}',      '(',      ')',      '->',     '+',      '-',
    '*',      '/',      '=',      '==',     '<',      '<=',
    '>',      'println','ID',     'NUM',
]
# Digit tokens replace NUM when BPE_DIGITS=1 (M3.7): numbers become spellable
# digit-by-digit (e.g. 42 -> [4,2]), so the model can reproduce literals.
DIGITS = [str(d) for d in range(10)]
LOWER = [chr(ord('a') + i) for i in range(26)]
UPPER = [chr(ord('A') + i) for i in range(26)]


def collect_corpus_text():
    parts = []
    include_big = os.environ.get('BPE_INCLUDE_BIG', '0') == '1'
    for root in ('kenga', 'examples'):
        for r, ds, fs in os.walk(root):
            for f in fs:
                if not f.endswith('.kenga'): continue
                if not include_big and f in SKIP_BIG: continue
                if f.startswith('mid_prophet') or f.startswith('pico_birth'): continue
                p = os.path.join(r, f)
                if 'kenga_seed_' in p: continue
                try:
                    parts.append(open(p, encoding='utf-8', errors='replace').read())
                except Exception:
                    pass
    return parts


def strip_comments(text):
    return re.sub(r'//[^\n]*', '', text)


def strip_strings(text):
    """Remove string/char literals. The 28-token tokenizer ignores string
    contents, so the codec must too (keeps representation consistent)."""
    text = re.sub(r'"(?:[^"\\]|\\.)*"', '', text)
    text = re.sub(r"'(?:[^'\\]|\\.)*'", '', text)
    return text


def identifier_words(docs):
    """All identifier words (non-keyword) from corpus, preserving case/underscore."""
    words = []
    for doc in docs:
        for w in re.findall(r'[A-Za-z_][A-Za-z0-9_]*', doc):
            if w not in KEYWORDS:
                words.append(w)
    return words


class Codec:
    def __init__(self, nmrg=10, digits=False, full=False):
        self.nmrg = nmrg
        self.digits = digits or full
        self.full = full
        self.letters = list(LOWER)
        if full:
            self.letters += UPPER + ['_']
        self.merges = []            # (a, b) ordered
        self.merge_set = set()      # merged strings available for encoding
        self.syntax = [t for t in SYNTAX if t != 'NUM']
        self.tokens = self.syntax + (DIGITS if self.digits else []) + self.letters
        self.token_to_id = {t: i for i, t in enumerate(self.tokens)}
        self.idx_to_token = self.tokens

    # ---- learning ----
    def fit(self, words):
        """Learn nmrg merges over identifier words (greedy BPE)."""
        # filter to words fully representable by the alphabet
        ok_chars = set(self.letters) | set(DIGITS)
        toks = [list(w) for w in words if all(c in ok_chars for c in w)]
        for _ in range(self.nmrg):
            pairs = collections.Counter()
            for t in toks:
                for i in range(len(t) - 1):
                    pairs[(t[i], t[i+1])] += 1
            if not pairs:
                break
            (a, b), cnt = pairs.most_common(1)[0]
            self.merges.append((a, b))
            self.merge_set.add(a + b)
            new_toks = []
            for t in toks:
                out = []
                i = 0
                while i < len(t):
                    if i + 1 < len(t) and t[i] == a and t[i+1] == b:
                        out.append(a + b); i += 2
                    else:
                        out.append(t[i]); i += 1
                new_toks.append(out)
            toks = new_toks

    # ---- encoding ----
    def _spellable(self, w):
        return all(c in self.letters or c in DIGITS for c in w)

    def encode_word(self, w):
        """Encode an identifier word -> list of token ids (greedy)."""
        if w in KEYWORDS:
            return [self.token_to_id[w]]
        if not self._spellable(w):
            return [self.token_to_id['ID']]
        toks = list(w)
        changed = True
        while changed:
            changed = False
            i = 0
            while i < len(toks) - 1:
                merged = toks[i] + toks[i+1]
                if merged in self.merge_set:
                    toks[i:i+2] = [merged]
                    changed = True
                i += 1
        return [self.token_to_id[t] for t in toks]

    def finalize(self):
        """Build final vocab: syntax + [digits] + letters + merges."""
        self.tokens = self.syntax + (DIGITS if self.digits else []) \
            + self.letters + [a + b for (a, b) in self.merges]
        self.token_to_id = {t: i for i, t in enumerate(self.tokens)}
        self.id_to_token = self.tokens


def main():
    docs = [strip_strings(strip_comments(t)) for t in collect_corpus_text()]
    words = identifier_words(docs)
    print(f'corpus docs: {len(docs)}, identifier words: {len(words)}')

    N = int(os.environ.get('BPE_MERGES', 10))
    use_digits = os.environ.get('BPE_DIGITS', '0') == '1'
    use_full = os.environ.get('BPE_FULL', '0') == '1'
    tag = 'full' if use_full else ('digits' if use_digits else 'bpe')
    codec = Codec(nmrg=N, digits=use_digits, full=use_full)
    codec.fit(words)
    print(f'\nlearned {len(codec.merges)} merges:')
    for i, (a, b) in enumerate(codec.merges):
        print(f'  merge #{i}: {a!r}+{b!r} -> {a+b!r}')

    codec.finalize()
    print(f'\nfinal vocab = {len(codec.tokens)} tokens')

    n_id, n_spell = 0, 0
    dist = collections.Counter()
    for w in set(words):
        ids = codec.encode_word(w)
        if ids == [codec.token_to_id['ID']]:
            n_id += 1
        else:
            n_spell += 1
            dist[len(ids)] += 1
    print(f'identifiers fully spellable: {n_spell}, ID-fallback: {n_id}')
    print(f'spelling length distribution: {dict(sorted(dist.items()))}')

    seed_ids = ['a','add','b','double','fact','fib','halve','i','max','min',
                'mul','n','p','pow','prodto','r','s','sqr','sub','sumto','x','y','main']
    print('\nseed identifiers:')
    for w in seed_ids:
        ids = codec.encode_word(w)
        toks = [codec.id_to_token[i] for i in ids]
        print(f'  {w:<8} -> {toks}')

    os.makedirs('minds', exist_ok=True)
    with open(f'minds/kenga_{tag}_vocab.txt', 'w') as f:
        f.write(f'# {tag} vocab = {len(codec.tokens)} (learned from train corpus, {N} merges)\n')
        for i, t in enumerate(codec.tokens):
            f.write(f'{i}\t{t}\n')
    with open(f'minds/kenga_{tag}_merges.txt', 'w') as f:
        for a, b in codec.merges:
            f.write(f'{a}\t{b}\n')
    with open(f'minds/kenga_{tag}.pkl', 'wb') as f:
        pickle.dump({'tokens': codec.tokens, 'merges': codec.merges}, f)
    print(f'\nsaved minds/kenga_{tag}_vocab.txt, kenga_{tag}_merges.txt, kenga_{tag}.pkl')


if __name__ == '__main__':
    main()
