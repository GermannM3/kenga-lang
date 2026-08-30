# Genesis copies

A copy is not a LoRA on a shared base. It is a named identity plus a private
experience journal. Two copies that `learn` different programs get different
`experience_sha1`. Fine-tune of weights waits on the realgen gate (M6: closed).

```
python tools/genesis_loop.py born --name alice --parent m6
python tools/genesis_loop.py learn --copy alice --src examples/selfhost/assign_lab.kenga
python tools/genesis_loop.py born --name bob --parent m6
python tools/genesis_loop.py learn --copy bob --src examples/selfhost/bitops.kenga
python tools/genesis_loop.py copies
python tools/genesis_loop.py grow --copy alice --parent m6
python tools/genesis_loop.py gate --parent m6
```

Journals live in `copies/<id>/` (gitignored). This README stays.
