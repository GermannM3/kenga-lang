# Backlog / cross-lane asks

## Z-lane (GPU machine)
- [ ] LoRA baseline @203M, rank 16, same WikiText-2 corpus — compare
      perplexity + train-time vs Z-Curriculum (killer feature if beaten)
- [ ] Token-level benchmark (WikiText-2 token-level or LLaMA-2 1.3B probe)
      — char-level is honest but not a standard
- [ ] Benchmarks on 5–7B scale when GPU budget allows (user directive)
- [ ] Measure encode/decode overhead numbers for pitch slide 6

## Kenga-lane (this machine, CPU)
- [ ] M6 evals → Genesis gate verdict (auto: M6_REPORT.md)
- [ ] If match T1 in 7–10%: Genesis dry-run (buffer accumulation, no train)
- [ ] If OPEN: branch A (Base control) first, then B/C/D
- [ ] HF publish M6 + rp1 after four logs (card: A/B rp0→rp1, NL axis,
      K=512 effect, ladder table)
- [ ] Language lane: lexer P0 fixes (`.` dropped, `&|^~`→ID, `=>` split),
      parser_laxity tests un-ignore, dialect inconsistencies list
      (SPEC agent report) → parser.rs owners
- [ ] Video walkthrough (bus factor mitigation) — after Genesis verdict
