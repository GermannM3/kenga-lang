"""tools/build_m6_report.py — assemble M6 gate verdicts into M6_REPORT.md."""
import re, json, time

def read(p):
    try:
        return open(p, encoding='utf-8', errors='replace').read()
    except OSError:
        return ''

train = read('train_m6.log')
held = re.findall(r'held (.+?): (\d+)/(\d+) = ([\d.]+)%', train)
overall = re.search(r'overall: (\d+)/(\d+) = ([\d.]+)%', train)
steps = re.findall(r'step\s+(\d+): batch-train-acc = ([\d.]+)%.*?loss = ([\d.]+)', train)

def total(path):
    t = read(path)
    m = re.search(r'TOTAL\s+\d+/\d+\s+\d+/(\d+)\s+(\d+)/\1\s+(\d+)/\1', t)
    pct = re.findall(r'(compile|match\(greedy\)|match\(pass@\d\)) ([\d.]+)%', t)
    n = re.search(r'programs=(\d+)|mutants=(\d+)|n=(\d+)', t)
    return m, pct, (n.group(0) if n else '')

nl_t = read('minds/corpus_factory/eval_m6_nl.log')
bind_t = read('minds/corpus_factory/eval_m6_bind.log')
fact_t = read('minds/corpus_factory/eval_m6_factory.log')
rg_t = read('minds/corpus_factory/eval_m6_realgen.log')

t1 = re.search(r'TIER 1 .*?n=(\d+)\n\s+compile (\d+)/\1 \((\d+)%\)\s+'
               r'semantic match (\d+)/\1 \((\d+)%\)\s+match pass@\d+ (\d+)/\1 \((\d+)%\)',
               rg_t, re.S)
t2 = re.search(r'TIER 2 .*?n=(\d+)\n\s+valid-program rate (\d+)/\1 \((\d+)%\)', rg_t, re.S)

lines = ['# M6 morning report', '', f'generated: {time.strftime("%Y-%m-%d %H:%M")}', '']
lines.append('## Training')
if steps:
    lines.append(f'first step {steps[0][0]}: acc {steps[0][1]}% loss {steps[0][2]}')
    lines.append(f'last step {steps[-1][0]}: acc {steps[-1][1]}% loss {steps[-1][2]}')
    first_loss, last_loss = float(steps[0][2]), float(steps[-1][2])
    lines.append(f'loss trend: {"OK (falling/stable)" if last_loss <= first_loss*1.15 else "WARNING: rising"}')
    lines.append('')
lines.append('### held-out (template-split factory + real files)')
for name, c, t_, p in held:
    lines.append(f'- {name}: {c}/{t_} = {p}%')
if overall:
    lines.append(f'- **overall: {overall.group(1)}/{overall.group(2)} = {overall.group(3)}%**')
lines.append('')
for title, txt in (('NL->code', nl_t), ('Bind', bind_t), ('Factory', fact_t)):
    tail = [l for l in txt.splitlines() if l.strip()][-4:]
    lines.append(f'## {title}')
    lines += ['```'] + tail + ['```', '']
lines.append('## Realgen v2')
if t1:
    lines.append(f'- Tier 1: n={t1.group(1)}, compile {t1.group(2)}% '
                 f'(gate >=30%: {"PASS" if int(t1.group(3))>=30 else "FAIL"}), '
                 f'match {t1.group(5)}% '
                 f'(gate >=10%: {"PASS" if int(t1.group(5))>=10 else "FAIL"}), '
                 f'match pass@k {t1.group(7)}%')
else:
    lines.append('- Tier 1: parse failed, see eval_m6_realgen.log')
if t2:
    lines.append(f'- Tier 2: valid-program rate {t2.group(2)}% (n={t2.group(1)})')
lines.append('')
b_c = re.search(r'compile ([\d.]+)%', bind_t)
lines.append('## Genesis gates verdict')
gates = []
if b_c:
    gates.append(('binding compile >=40%',
                  'PASS' if float(b_c.group(1)) >= 40 else 'FAIL', b_c.group(1)+'%'))
if t1:
    gates.append(('realgen T1 compile >=30%',
                  'PASS' if int(t1.group(3)) >= 30 else 'FAIL', t1.group(3)+'%'))
    gates.append(('realgen T1 match >=10%',
                  'PASS' if int(t1.group(5)) >= 10 else 'FAIL', t1.group(5)+'%'))
for name, v, x in gates:
    lines.append(f'- [{"x" if v=="PASS" else " "}] {name}: {x} -> {v}')
allp = all(v == 'PASS' for _, v, _ in gates) and len(gates) == 3
lines.append('')
lines.append('**GENESIS: OPEN — start branch A**' if allp and gates else
             '**GENESIS: CLOSED — see failing gates above**')
open('minds/corpus_factory/M6_REPORT.md', 'w', encoding='utf-8').write('\n'.join(lines))
print('report written')