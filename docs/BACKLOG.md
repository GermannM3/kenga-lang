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
- [x] Live dialog M5.3 x `kenga-trained` (Ollama) → verified buffer, extra_dir absorb, no D/L grow (`tools/live_dialog.py`)
- [ ] If match T1 in 7–10%: wire `finetune` / `grow --train` (still no LoRA)
- [ ] If OPEN: branch A (Base control) first, then B/C/D
- [ ] HF publish M6 + rp1 after four logs
- [x] Z-lane Etap 2 (identity): copies stamp parent passport marker (`genesis_loop.py verify`); D8 `same_agent(L)` + `--verdict` (M6 ≠ M5.3 lineage)
- [x] Z identity in guest Kenga: `z_is_alive`/`z_verify` = file_exists + `.passport.json` (no OP_*, no SVD in VM); lexer keywords; factory `--z1` pilot (`pilot_z1.jsonl`, not split_v3)
- [ ] Z-lane Etap 2+: Factory Z2–Z5, Genesis spectral `z_verify` on grow (Python zcore)
- [ ] Language lane: lexer P0 fixes (`.` dropped, `&|^~`→ID, `=>` split),
      parser_laxity tests un-ignore, dialect inconsistencies list
      (SPEC agent report) → parser.rs owners
- [ ] Video walkthrough (bus factor mitigation) — after Genesis verdict
