from pathlib import Path

last = Path("/opt/kenga-lang/minds/tg_last.txt")
last.write_text("", encoding="utf-8")

p = Path("/opt/kenga-lang/minds/tg_pairs.txt")
raw = p.read_text(encoding="utf-8", errors="replace")
junk = ("Саше", "понравил", "Завис", "сентябр", "подготов", "Посикун")
keep = []
for line in raw.splitlines():
    if "|" not in line:
        continue
    if any(j in line for j in junk):
        continue
    a, b = line.split("|", 1)
    if a.strip() == b.strip():
        continue
    keep.append(line)
p.write_text(("\n".join(keep) + "\n") if keep else "", encoding="utf-8")
print("kept", len(keep))
