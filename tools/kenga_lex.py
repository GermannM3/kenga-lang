"""Shared Kenga codec lexer (Prophet tokenize).

Language scanners in kenga/ already keep '.' vs '..' (field vs range) and
do not silently drop them. Python codec lexers used to:

  * drop '.' (so 0..10 became digits with no range marker)
  * put 1-char bitwise ops in TWO_CHAR (never matched mid-source) and map
    real 2-char ops missing from vocab (logical-and, shifts) to ID
  * split => into = > and :: into two colons

Vocab 128 (minds/kenga_full.pkl) has no '.' '..' '=>' '::' '&' '|' '^' '~'.
emit_op never remaps those to ID. They are a documented hole until the
codec is retrained; lex_raw still yields the glyphs.
"""

DEFAULT_KEYWORDS = frozenset({
    'fn', 'return', 'let', 'if', 'else', 'while', 'for', 'i64', 'println',
})

# Longest-match 2-char ops. 1-char bitwise ops do NOT belong here.
TWO_CHAR_OPS = frozenset({
    '->', '=>', '::', '..',
    '==', '<=', '>=', '!=',
    '&&', '||', '<<', '>>',
})

ONE_CHAR_OPS = frozenset(':,;{}()+-*/=<>.&|^~')


def lex_raw(src, keep_comments=False):
    """Yield (kind, text) lexemes. kind is 'op', 'num', or 'word'.

    Unknown glyphs are skipped (same as before) but operators in
    TWO_CHAR_OPS / ONE_CHAR_OPS are never dropped without a lexeme.
    """
    out = []
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        if c in ' \t\n\r':
            i += 1
            continue
        if c == '/' and i + 1 < n and src[i + 1] == '/':
            if not keep_comments:
                while i < n and src[i] != '\n':
                    i += 1
                continue
            i += 2
            j = i
            while j < n and src[j] != '\n':
                if src[j].isalnum() or src[j] == '_':
                    e = j
                    while e < n and (src[e].isalnum() or src[e] == '_'):
                        e += 1
                    out.append(('word', src[j:e]))
                    j = e
                else:
                    j += 1
            i = j
            continue
        two = src[i:i + 2]
        if two in TWO_CHAR_OPS:
            out.append(('op', two))
            i += 2
            continue
        if c in ONE_CHAR_OPS:
            out.append(('op', c))
            i += 1
            continue
        if c.isdigit():
            j = i
            while j < n and src[j].isdigit():
                j += 1
            out.append(('num', src[i:j]))
            i = j
            continue
        if c.isalpha() or c == '_':
            j = i
            while j < n and (src[j].isalnum() or src[j] == '_'):
                j += 1
            out.append(('word', src[i:j]))
            i = j
            continue
        i += 1
    return out


def emit_op(token_to_id, tok, out):
    """Append vocab id for a syntax lexeme. Never remap unknown ops to ID."""
    vid = token_to_id.get(tok)
    if vid is not None:
        out.append(vid)
        return True
    return False


def tokens_to_ids(raw, token_to_id, encode_word=None, keywords=None):
    if keywords is None:
        keywords = DEFAULT_KEYWORDS
    out = []
    has_num = 'NUM' in token_to_id
    id_tok = token_to_id.get('ID')
    for kind, text in raw:
        if kind == 'op':
            emit_op(token_to_id, text, out)
        elif kind == 'num':
            if has_num:
                out.append(token_to_id['NUM'])
            else:
                for d in text:
                    if d in token_to_id:
                        out.append(token_to_id[d])
                    elif id_tok is not None:
                        # digit missing from this vocab: last resort for NUM
                        out.append(id_tok)
        elif kind == 'word':
            if encode_word is not None:
                out.extend(encode_word(text))
            elif text in keywords and text in token_to_id:
                out.append(token_to_id[text])
            elif id_tok is not None:
                out.append(id_tok)
    return out


def tokenize_src(src, token_to_id, encode_word=None, keep_comments=False,
                 keywords=None):
    return tokens_to_ids(
        lex_raw(src, keep_comments=keep_comments),
        token_to_id,
        encode_word=encode_word,
        keywords=keywords,
    )
