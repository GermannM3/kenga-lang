"""tools/kenchat_cli.py — interactive chat/coding shell for Kenga Prophet.

Usage:
    python tools/kenchat_cli.py --model m37
    python tools/kenchat_cli.py --model m37 --chat "fn add"
    python tools/kenchat_cli.py --model m37 --file examples/ml/kenga_seed_add.kenga

Modes:
  - interactive: read commands from stdin, generate + run each.
  - --chat "QUERY": single-shot generate-and-run, print program + output.
  - --file PATH:  open a .kenga file, autocomplete the next block, run it.

Commands in interactive mode:
  gen <prompt>      generate a program from a code prefix / description
  run <prompt>      generate, wrap into a valid program, execute via kenga-lite
  verify <n>        run self-consistency (n candidates), pick first that runs
  hist <n>          include last n generated programs as context (default 0)
  quit / exit       leave the shell
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kenchat


MODELS = {
    'm37': ('minds/mid_prophet_m37_w.txt', 'minds/kenga_digits.pkl'),
    'm40': ('minds/mid_prophet_m40_w.txt', 'minds/kenga_full.pkl'),
    'm41': ('minds/mid_prophet_m41_w.txt', 'minds/kenga_full.pkl'),
    'm42': ('minds/mid_prophet_m42_w.txt', 'minds/kenga_full.pkl'),
}


def run_prompt(model, weights, codec, prompt, max_tokens=200, verify=0,
               temperature=1.0, history=None):
    """Generate a program for prompt, optionally verify, run it, print."""
    ctx = ''
    if history:
        ctx = '\n'.join(history[-3:]) + '\n'
    full_prompt = (ctx + prompt).strip()
    if verify:
        toks, src, full, rc, out, err = kenchat.gen_verified(
            full_prompt, weights, codec, n_samples=verify,
            max_tokens=max_tokens, temperature=temperature)
    else:
        toks, src = kenchat.gen_tokens(full_prompt, weights, max_tokens=max_tokens,
                                       temperature=None, codec=codec)
        full = kenchat.make_valid_program(full_prompt, src)
        rc, out, err = kenchat.run_via_kenga_lite(full)
    print('--- generated ---')
    print(full)
    print('--- run ---')
    if out:
        print('stdout:')
        print(out.rstrip())
    if err:
        print('stderr:', err.strip()[:200])
    print(f'rc={rc}')
    return full, rc, out, err


def interactive(model, weights, codec):
    print(f'Kenga Prophet chat — model {model}')
    print('Commands: gen/run <prompt>, verify <n> <prompt>, hist <n>, quit')
    print('Example: run "fn add"  or  verify 8 "fn sqr"')
    print('Enter a bare prompt to run it with self-consistency (verify 8).')
    history = []
    hist_n = 0
    while True:
        try:
            line = input('\n>>> ').strip()
        except (EOFError, KeyboardInterrupt):
            break
        if not line:
            continue
        low = line.lower()
        if low in ('quit', 'exit', 'q'):
            break
        if low.startswith('hist '):
            hist_n = int(low.split()[1])
            print(f'context window: last {hist_n} programs')
            continue
        prompt = line
        verify = 0
        if low.startswith('verify '):
            parts = line.split(None, 2)
            verify = int(parts[1])
            prompt = parts[2] if len(parts) > 2 else ''
        elif low.startswith('gen '):
            prompt = line[4:]
        elif low.startswith('run '):
            prompt = line[4:]
        else:
            verify = 8
        if not prompt:
            print('(empty prompt)')
            continue
        full, rc, out, err = run_prompt(model, weights, codec, prompt,
                                        max_tokens=200, verify=verify,
                                        history=history if hist_n else None)
        history.append(full)
        if len(history) > 20:
            history = history[-20:]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--model', default='m37', choices=list(MODELS))
    ap.add_argument('--chat', default=None, help='single-shot prompt')
    ap.add_argument('--verify', type=int, default=8,
                    help='self-consistency samples for single-shot (default 8)')
    ap.add_argument('--file', default=None, help='autocomplete a .kenga file')
    ap.add_argument('--max-tokens', type=int, default=200)
    args = ap.parse_args()

    if args.model not in MODELS:
        print('unknown model', file=sys.stderr)
        return 1
    weights, codec_path = MODELS[args.model]
    codec = kenchat.load_codec_vocab(codec_path)

    if args.file:
        with open(args.file, encoding='utf-8') as f:
            src = f.read()
        toks = kenchat.tokenize(src, codec)
        if len(toks) > 30:
            # trim to last ~28 tokens as autocomplete prefix
            toks = toks[-28:]
        prompt = kenchat.detokenize(toks, codec)
        print(f'autocomplete from {args.file} (prefix: {prompt[:80]}...)')
        run_prompt(args.model, weights, codec, prompt, args.max_tokens,
                   args.verify)
        return 0

    if args.chat:
        run_prompt(args.model, weights, codec, args.chat, args.max_tokens,
                   args.verify)
        return 0

    interactive(args.model, weights, codec)
    return 0


if __name__ == '__main__':
    sys.exit(main())
