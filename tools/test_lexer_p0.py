"""P0 codec lexer: `.` `..` `=>` `::` `&|^~` must not silently vanish / become ID.

Run: python tools/test_lexer_p0.py
No pytest. Exits non-zero on failure.

Vocab 128 has no terminals for those glyphs, so encode cannot preserve them
without retraining. lex_raw must still emit them; map must not use ID.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenga_lex
import kenchat
import train_m3


def decode(ids, token_to_id):
    inv = {i: t for t, i in token_to_id.items()}
    return [inv.get(i, i) for i in ids]


def ops_of(src):
    return [t for k, t in kenga_lex.lex_raw(src) if k == 'op']


def main():
    failed = 0

    def check(name, cond, detail=''):
        nonlocal failed
        if cond:
            print('ok  ', name)
        else:
            failed += 1
            print('FAIL', name, detail)

    check('lex ..', ops_of('for i in 0..10') == ['..'])
    check('lex .', ops_of('obj.field') == ['.'])
    check('lex =>', ops_of('x => y') == ['=>'])
    check('lex ::', ops_of('A::B') == ['::'])
    check('lex bitwise', ops_of('a & b | c ^ d ~ e') == ['&', '|', '^', '~'])
    check('lex << >>', ops_of('1 << 2 >> 3') == ['<<', '>>'])
    check('lex -> still', ops_of('fn main() -> i64') == ['(', ')', '->'])

    check('=> not split', kenga_lex.lex_raw('x=>y') == [
        ('word', 'x'), ('op', '=>'), ('word', 'y'),
    ])
    check(':: not split', kenga_lex.lex_raw('A::B') == [
        ('word', 'A'), ('op', '::'), ('word', 'B'),
    ])
    check('.. not two dots', kenga_lex.lex_raw('0..10') == [
        ('num', '0'), ('op', '..'), ('num', '10'),
    ])

    codec_path = os.path.join(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
        'minds', 'kenga_full.pkl')
    codec = kenchat.load_codec_vocab(codec_path)
    t2i = codec['token_to_id']
    ID = t2i['ID']
    check('V==128', len(codec['tokens']) == 128, str(len(codec['tokens'])))

    for h in ['.', '..', '=>', '::', '&', '|', '^', '~']:
        check('hole ' + h, h not in t2i)

    tok = kenchat.tokenize
    check('& not ID', ID not in tok('a & b', codec))
    check('| not ID', ID not in tok('a | b', codec))
    check('^ not ID', ID not in tok('a ^ b', codec))
    check('~ not ID', ID not in tok('~0', codec))
    check('&& not ID', ID not in tok('a && b', codec))
    check('<< not ID', ID not in tok('1 << 2', codec))

    eq, gt = t2i['='], t2i['>']
    ids = tok('x => y', codec)
    check('=> not = >', eq not in ids and gt not in ids, decode(ids, t2i))

    colon = t2i[':']
    ids = tok('A::B', codec)
    check(':: not two :', ids.count(colon) == 0, decode(ids, t2i))

    ids = tok('for i in 0..10', codec)
    check('range no ID', ID not in ids, decode(ids, t2i))
    check('range keeps 0 1 0', decode(ids, t2i)[-3:] == ['0', '1', '0'],
          decode(ids, t2i))

    ids28 = train_m3.tokenize('x => y')
    voc = train_m3.VOCAB
    check('m3 => not = >', voc['='] not in ids28 and voc['>'] not in ids28)

    t3 = train_m3.make_codec_tokenize(codec)
    check('m3 codec => not = >',
          t2i['='] not in t3('x => y') and t2i['>'] not in t3('x => y'))

    if failed:
        print(failed, 'failed')
        sys.exit(1)
    print('lexer P0 ok')


if __name__ == '__main__':
    main()
