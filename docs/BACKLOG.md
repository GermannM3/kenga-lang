# Backlog / cross-lane asks

## Z-lane (GPU machine)
- [ ] LoRA baseline @203M, rank 16, same WikiText-2 corpus — compare
      perplexity + train-time vs Z-Curriculum (killer feature if beaten)
- [ ] Token-level benchmark (WikiText-2 token-level or LLaMA-2 1.3B probe)
      — char-level is honest but not a standard
- [ ] Benchmarks on 5–7B scale when GPU budget allows (user directive)
- [ ] Measure encode/decode overhead numbers for pitch slide 6

## Kenga-lane (this machine, CPU)
- [x] Z x Kenga Etap 0 (corpus_eval prog->_prog) + Etap 1: zcore.py (11 ops), 14/14 unit tests, passports for m53/m6 (.passport.json), test_zcore.py
- [ ] If match T1 in 7–10%: Genesis dry-run (buffer accumulation, no train)
- [ ] If OPEN: branch A (Base control) first, then B/C/D
- [ ] HF publish M6 + rp1 after four logs
- [ ] Z-lane Etap 2+: lexer z_* keywords, VM builtins, Factory Z1-Z5, Genesis z_verify gate (spec v0.1+v0.2) (card: A/B rp0→rp1, NL axis,
      K=512 effect, ladder table)
- [ ] Language lane: lexer P0 fixes (`.` dropped, `&|^~`→ID, `=>` split),
      parser_laxity tests un-ignore, dialect inconsistencies list
      (SPEC agent report) → parser.rs owners
- [ ] Video walkthrough (bus factor mitigation) — after Genesis verdict
