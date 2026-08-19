"""Check corpus tokens."""
import os

KEYWORDS = {'fn','return','let','if','else','while','for','i64','println'}
TWO_CHAR = {'->','==','<=','>='}

def tokenize(src):
    out = []
    i = 0
    while i < len(src):
        c = src[i]
        if c in (' ', '\t', '\n', '\r'):
            i += 1
            continue
        if c == '/' and i+1 < len(src) and src[i+1] == '/':
            while i < len(src) and src[i] != '\n':
                i += 1
            continue
        two = src[i:i+2]
        if two in TWO_CHAR:
            out.append(two); i += 2; continue
        if c in (':', ',', ';', '{', '}', '(', ')', '+', '-', '*', '/', '=', '<', '>'):
            if c == '-' and i+1 < len(src) and src[i+1] == '>':
                out.append('->'); i += 2; continue
            out.append(c); i += 1; continue
        if c.isdigit():
            j = i
            while j < len(src) and src[j].isdigit(): j += 1
            out.append('NUM'); i = j; continue
        if c.isalpha() or c == '_':
            j = i
            while j < len(src) and (src[j].isalnum() or src[j] == '_'):
                j += 1
            word = src[i:j]
            if word in KEYWORDS:
                out.append(word)
            else:
                out.append('ID')
            i = j; continue
        i += 1
    return out


total_tokens = 0
files = []
for root in ('kenga', 'examples'):
    for r, ds, fs in os.walk(root):
        for f in fs:
            if f.endswith('.kenga'):
                p = os.path.join(r, f)
                try:
                    data = open(p, encoding='utf-8', errors='replace').read()
                    tok = tokenize(data)
                    total_tokens += len(tok)
                    files.append((p, len(tok)))
                except Exception:
                    pass
files.sort(key=lambda x: -x[1])
print(f'total files: {len(files)}, total tokens: {total_tokens}')
print('top-10:')
for p, n in files[:10]:
    print(f'  {n:6d}  {p}')
