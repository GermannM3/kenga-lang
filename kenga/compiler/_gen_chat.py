# ASCII-only generator: writes chat.kenga as UTF-8.
# Run: python kenga/compiler/_gen_chat.py

from pathlib import Path


def u(s: str) -> str:
    return s.encode("ascii").decode("unicode_escape")


W = {
    "privet": u("\\u043f\\u0440\\u0438\\u0432\\u0435\\u0442"),
    "vyhod": u("\\u0432\\u044b\\u0445\\u043e\\u0434"),
    "poka": u("\\u043f\\u043e\\u043a\\u0430"),
    "zdrav": u("\\u0437\\u0434\\u0440\\u0430\\u0432\\u0441\\u0442\\u0432"),
    "chto_umeesh": u("\\u0447\\u0442\\u043e \\u0443\\u043c\\u0435\\u0435\\u0448\\u044c"),
    "chto_ty_umeesh": u("\\u0447\\u0442\\u043e \\u0442\\u044b \\u0443\\u043c\\u0435\\u0435\\u0448\\u044c"),
    "chto_mozhesh": u("\\u0447\\u0442\\u043e \\u043c\\u043e\\u0436\\u0435\\u0448\\u044c"),
    "vozmozhnosti": u("\\u0432\\u043e\\u0437\\u043c\\u043e\\u0436\\u043d\\u043e\\u0441\\u0442\\u0438"),
    "pogovori": u("\\u043f\\u043e\\u0433\\u043e\\u0432\\u043e\\u0440\\u0438"),
    "pogovorim": u("\\u043f\\u043e\\u0433\\u043e\\u0432\\u043e\\u0440\\u0438\\u043c"),
    "prosto_pog": u("\\u043f\\u0440\\u043e\\u0441\\u0442\\u043e \\u043f\\u043e\\u0433\\u043e\\u0432\\u043e\\u0440\\u0438\\u0442\\u044c"),
    "hochu_pog": u("\\u0445\\u043e\\u0447\\u0443 \\u043f\\u043e\\u0433\\u043e\\u0432\\u043e\\u0440\\u0438\\u0442\\u044c"),
    "davay_bol": u("\\u0434\\u0430\\u0432\\u0430\\u0439 \\u0431\\u043e\\u043b\\u0442\\u0430\\u0442\\u044c"),
    "ty_kto": u("\\u0442\\u044b \\u043a\\u0442\\u043e"),
    "kto_ty": u("\\u043a\\u0442\\u043e \\u0442\\u044b"),
    "rasskazhi": u("\\u0440\\u0430\\u0441\\u0441\\u043a\\u0430\\u0436\\u0438 \\u043e \\u0441\\u0435\\u0431\\u0435"),
    "o_sebe": u("\\u043e \\u0441\\u0435\\u0431\\u0435"),
    "chto_ty_takoe": u("\\u0447\\u0442\\u043e \\u0442\\u044b \\u0442\\u0430\\u043a\\u043e\\u0435"),
    "chestno": u("\\u0447\\u0435\\u0441\\u0442\\u043d\\u043e"),
    "ty_ii": u("\\u0442\\u044b \\u0438\\u0438"),
    "ty_llm": u("\\u0442\\u044b llm"),
    "kak_grok": u("\\u043a\\u0430\\u043a grok"),
    "kak_gpt": u("\\u043a\\u0430\\u043a gpt"),
    "neyro": u("\\u043d\\u0435\\u0439\\u0440\\u043e\\u0441\\u0435\\u0442\\u044c"),
    "yaz_mod": u("\\u044f\\u0437\\u044b\\u043a\\u043e\\u0432\\u0430\\u044f \\u043c\\u043e\\u0434\\u0435\\u043b\\u044c"),
    "eto_vse": u("\\u044d\\u0442\\u043e \\u0432\\u0441\\u0451"),
    "pomoshch": u("\\u043f\\u043e\\u043c\\u043e\\u0449\\u044c"),
    "status": u("\\u0441\\u0442\\u0430\\u0442\\u0443\\u0441"),
    "kak_dela": u("\\u043a\\u0430\\u043a \\u0434\\u0435\\u043b\\u0430"),
    "spi": u("\\u0441\\u043f\\u0438"),
    "konsolid": u("\\u043a\\u043e\\u043d\\u0441\\u043e\\u043b\\u0438\\u0434"),
    "zasni": u("\\u0437\\u0430\\u0441\\u043d\\u0438"),
    "sohrani": u("\\u0441\\u043e\\u0445\\u0440\\u0430\\u043d\\u0438"),
    "zapishi": u("\\u0437\\u0430\\u043f\\u0438\\u0448\\u0438 mind"),
    "zagruzi": u("\\u0437\\u0430\\u0433\\u0440\\u0443\\u0437\\u0438"),
    "obuchi": u("\\u043e\\u0431\\u0443\\u0447\\u0438"),
    "trenir": u("\\u0442\\u0440\\u0435\\u043d\\u0438\\u0440"),
    "uchis": u("\\u0443\\u0447\\u0438\\u0441\\u044c"),
    "budush": u("\\u0431\\u0443\\u0434\\u0443\\u0449"),
    "cherez": u("\\u0447\\u0435\\u0440\\u0435\\u0437"),
    "chto_budet": u("\\u0447\\u0442\\u043e \\u0431\\u0443\\u0434\\u0435\\u0442"),
    "predskazhi": u("\\u043f\\u0440\\u0435\\u0434\\u0441\\u043a\\u0430\\u0436\\u0438"),
    "zavtra": u("\\u0437\\u0430\\u0432\\u0442\\u0440\\u0430"),
    "skoro": u("\\u0441\\u043a\\u043e\\u0440\\u043e"),
    "dalshe": u("\\u0434\\u0430\\u043b\\u044c\\u0448\\u0435"),
    "smotri": u("\\u0441\\u043c\\u043e\\u0442\\u0440\\u0438"),
    "nablyud": u("\\u043d\\u0430\\u0431\\u043b\\u044e\\u0434"),
    "seychas": u("\\u0441\\u0435\\u0439\\u0447\\u0430\\u0441"),
    "sostoyanie": u("\\u0441\\u043e\\u0441\\u0442\\u043e\\u044f\\u043d\\u0438\\u0435"),
    "nauchi": u("\\u043d\\u0430\\u0443\\u0447\\u0438"),
    "zapomni": u("\\u0437\\u0430\\u043f\\u043e\\u043c\\u043d\\u0438"),
    "vyuchi": u("\\u0432\\u044b\\u0443\\u0447\\u0438"),
    "vspomni": u("\\u0432\\u0441\\u043f\\u043e\\u043c\\u043d\\u0438"),
    "pohozh": u("\\u043f\\u043e\\u0445\\u043e\\u0436"),
    "arrow": u("\\u2192"),
    "laquo": u("\\u00ab"),
    "raquo": u("\\u00bb"),
    "mdash": u("\\u2014"),
    "hellip": u("\\u2026"),
    "bullet": u("\\u2022"),
}

src = r'''// Prophet chat on native_ml lists. Host CRT: file_exists / read_line / argc.
// kenga-lite chat [mind.km] [--script file] -> this file

import "native_ml.kenga"

fn has(hay: str, needle: str) -> i64 {
    let n: i64 = len(needle);
    let m: i64 = len(hay);
    if n < 1 { return 0; }
    if n > m { return 0; }
    let i: i64 = 0;
    while i + n <= m {
        let ok: i64 = 1;
        let j: i64 = 0;
        while j < n {
            if hay[i + j] != needle[j] { ok = 0; }
            j = j + 1;
        }
        if ok == 1 { return 1; }
        i = i + 1;
    }
    return 0;
}

fn has_any(hay: str, xs: list) -> i64 {
    let i: i64 = 0;
    while i < len(xs) {
        if has(hay, xs[i]) == 1 { return 1; }
        i = i + 1;
    }
    return 0;
}

fn ascii_lower(s: str) -> str {
    let abc: str = "abcdefghijklmnopqrstuvwxyz";
    let out: str = "";
    let i: i64 = 0;
    while i < len(s) {
        let o: i64 = ord(s[i]);
        if o >= 65 {
            if o <= 90 {
                out = out + abc[o - 65];
                i = i + 1;
                continue;
            }
        }
        out = out + s[i];
        i = i + 1;
    }
    return out;
}

fn trim(s: str) -> str {
    let a: i64 = 0;
    let b: i64 = len(s);
    while a < b {
        let o: i64 = ord(s[a]);
        if o == 32 { a = a + 1; continue; }
        if o == 9 { a = a + 1; continue; }
        break;
    }
    while b > a {
        let o: i64 = ord(s[b - 1]);
        if o == 32 { b = b - 1; continue; }
        if o == 9 { b = b - 1; continue; }
        if o == 13 { b = b - 1; continue; }
        if o == 10 { b = b - 1; continue; }
        break;
    }
    let out: str = "";
    let i: i64 = a;
    while i < b {
        out = out + s[i];
        i = i + 1;
    }
    return out;
}

fn is_num_start(o: i64) -> i64 {
    if o == 45 { return 1; }
    if o == 46 { return 1; }
    if o >= 48 {
        if o <= 57 { return 1; }
    }
    return 0;
}

fn extract_nums(s: str) -> list {
    let xs: list = [];
    let i: i64 = 0;
    let n: i64 = len(s);
    while i < n {
        if is_num_start(ord(s[i])) == 0 {
            i = i + 1;
            continue;
        }
        let tok: str = "";
        let j: i64 = i;
        let digits: i64 = 0;
        while j < n {
            let c: i64 = ord(s[j]);
            let ok: i64 = 0;
            if c == 45 { ok = 1; }
            if c == 43 { ok = 1; }
            if c == 46 { ok = 1; }
            if c == 101 { ok = 1; }
            if c == 69 { ok = 1; }
            if c >= 48 {
                if c <= 57 {
                    ok = 1;
                    digits = 1;
                }
            }
            if ok == 0 { break; }
            tok = tok + s[j];
            j = j + 1;
        }
        if digits == 1 { xs = push(xs, nt_parse_f(tok)); }
        if j == i { i = i + 1; } else { i = j; }
    }
    return xs;
}

fn is_num_line(s: str) -> i64 {
    if len(s) < 1 { return 0; }
    let i: i64 = 0;
    while i < len(s) {
        let o: i64 = ord(s[i]);
        let ok: i64 = 0;
        if o == 32 { ok = 1; }
        if o == 9 { ok = 1; }
        if o == 45 { ok = 1; }
        if o == 43 { ok = 1; }
        if o == 46 { ok = 1; }
        if o == 101 { ok = 1; }
        if o == 69 { ok = 1; }
        if o >= 48 {
            if o <= 57 { ok = 1; }
        }
        if ok == 0 { return 0; }
        i = i + 1;
    }
    return 1;
}

fn take(xs: list, a: i64, b: i64) -> list {
    let o: list = [];
    let i: i64 = a;
    while i < b {
        o = push(o, xs[i]);
        i = i + 1;
    }
    return o;
}

fn fmt_round(xs: list) -> str {
    let s: str = "";
    let i: i64 = 0;
    while i < len(xs) {
        if i > 0 { s = s + " "; }
        s = s + to_str(round(xs[i] + 0.0));
        i = i + 1;
    }
    return s;
}

fn print_help() -> i64 {
    println("talk:     {privet} | {ty_kto} | {chto_umeesh} | {pogovorim} | {chestno}");
    println("model:    {smotri} 5 1 6 | {chto_budet} {cherez} 4 | {obuchi} 10");
    println("memory:   {nauchi} 5 1 6 -> 6 1 5 | {vspomni} {hellip} | {spi} | {sohrani}");
    println("quit:     {vyhod} / quit");
    println("(native_ml, not C Prophet)");
    return 0;
}

fn open_mind(path: str) -> list {
    if file_exists(path) == 0 {
        println("new mind -> will save to " + path);
        return nt_mind(0.05, 128, 48);
    }
    let s: str = read_file(path);
    let p = nt_read_tok(s, 0);
    if p[0] == "MORE_MIND" {
        println("loading " + path);
        return nt_load_mind(path);
    }
    println("old format " + path + " -- new mind (MORE_MIND)");
    return nt_mind(0.05, 128, 48);
}

fn phys_x(last: list) -> list {
    if len(last) >= 3 { return take(last, 0, 3); }
    return [5.0, 1.0, 6.0];
}

fn phys_y(x: list) -> list {
    return [x[0] + x[1], x[1] - 0.1, x[2] - 0.05];
}

fn train_epoch(m: list, last: list) -> f64 {
    let x = phys_x(last);
    return nt_learn(m, x, phys_y(x));
}

fn handle(st: list, line: str) -> i64 {
    let m = st[0];
    let last = st[1];
    let path: str = st[2];
    let low: str = ascii_lower(line);
    let nums = extract_nums(line);
    let nn: i64 = len(nums);

    if has_any(low, ["quit", "exit", "{vyhod}", "{poka}"]) == 1 {
        println("{poka} -- mind on disk if you saved");
        return 0;
    }
    if has_any(low, ["{privet}", "hello", "hi", "{zdrav}", "hey"]) == 1 {
        println("{privet}. world-model Kenga on native_ml (steps=" + to_str(m[6]) + ", dim=" + to_str(m[5]) + ").");
        if m[5] >= 9 {
            println("this mind is multimodal: image+sound -> vector -> predict.");
        } else {
            println("this mind is agent physics [pos vel fuel].");
        }
        println("try: {laquo}{chto_umeesh}{raquo}, {laquo}{pogovorim}{raquo}, {laquo}{smotri}{hellip}{raquo}, {laquo}{pomoshch}{raquo}.");
        return 1;
    }
    if has_any(low, ["{chto_umeesh}", "{chto_ty_umeesh}", "{chto_mozhesh}", "{vozmozhnosti}", "capabilities"]) == 1 {
        println("can do now (native_ml):");
        println("  {bullet} predict next vector (predict / unroll)");
        println("  {bullet} surprise + episodes; sleep (consolidate)");
        println("  {bullet} save/load MORE_MIND, not C Prophet");
        if m[5] >= 9 { println("  {bullet} multimodal obs dim=" + to_str(m[5])); }
        println("cannot: free chat like a big LLM.");
        return 1;
    }
    if has_any(low, ["{pogovori}", "{pogovorim}", "{prosto_pog}", "{hochu_pog}", "{davay_bol}"]) == 1 {
        println("ok. I talk about what the model body remembers.");
        println("now: steps=" + to_str(m[6]) + " dim=" + to_str(m[5]) + " ep=" + to_str(len(m[11])) + ".");
        println("for the ceiling -- {laquo}{chestno}{raquo}.");
        return 1;
    }
    if has_any(low, ["{ty_kto}", "{kto_ty}", "{rasskazhi}", "{o_sebe}", "what are you", "who are you", "{chto_ty_takoe}"]) == 1 {
        println("Prophet-mind: residual MLP, dim=" + to_str(m[5]) + ", hidden=" + to_str(m[3]) + ", steps=" + to_str(m[6]) + ".");
        println("episodic=" + to_str(len(m[11])) + ". sense->learn->sleep.");
        if m[5] >= 9 { println("trained on PPM+WAV (living multimodal)."); }
        println("not a chat-LLM -- loop on native_ml.");
        return 1;
    }
    if has_any(low, ["{chestno}", "{ty_ii}", "{ty_llm}", "{kak_grok}", "{kak_gpt}", "{neyro}", "{yaz_mod}", "{eto_vse}"]) == 1 {
        println("{chestno}: I am a world-model (vectors -> vectors), not a language model.");
        println("smart like Grok = tiny LM -> data -> f32/GPU. living loop is already here.");
        return 1;
    }
    if low == "?" {
        print_help();
        return 1;
    }
    if low == "help" {
        print_help();
        return 1;
    }
    if has(low, "{pomoshch}") == 1 {
        print_help();
        return 1;
    }
    if has_any(low, ["status", "{status}", "{kak_dela}", "stats"]) == 1 {
        let stt = nt_mem_stats(m);
        println("I remember: ep=" + to_str(stt[0]) + " steps=" + to_str(m[6]) + " dim=" + to_str(m[5]) + "x" + to_str(m[3]) + " (lr=" + to_str(m[4]) + ")");
        return 1;
    }
    if has_any(low, ["sleep", "{spi}", "{konsolid}", "{zasni}"]) == 1 {
        println("sleep: folded " + to_str(nt_consolidate(m)));
        return 1;
    }
    if has_any(low, ["save", "{sohrani}", "{zapishi}"]) == 1 {
        if nt_save_mind(m, path) == 1 {
            println("saved " + path);
        } else {
            println("could not save " + path);
        }
        return 1;
    }
    if has_any(low, ["load", "{zagruzi}"]) == 1 {
        st[0] = open_mind(path);
        println("loaded " + path);
        return 1;
    }
    if has_any(low, ["train", "{obuchi}", "{trenir}", "{uchis}"]) == 1 {
        let epochs: i64 = 8;
        if nn > 0 { epochs = round(nums[0]); }
        if epochs < 1 { epochs = 1; }
        if epochs > 64 { epochs = 64; }
        println("training " + to_str(epochs) + " epochs...");
        let e: i64 = 0;
        while e < epochs {
            let loss: f64 = train_epoch(m, last);
            println("  epoch " + to_str(e) + ": loss=" + to_str(loss));
            e = e + 1;
        }
        println("slept, folded episodes: " + to_str(nt_consolidate(m)));
        return 1;
    }
    if has_any(low, ["future", "{budush}", "{cherez}", "{chto_budet}", "unroll", "{predskazhi}", "{zavtra}", "{skoro}", "{dalshe}"]) == 1 {
        let steps: i64 = 4;
        let obs: list = [];
        if has(low, "{zavtra}") == 1 { steps = 1; }
        if has(low, "{skoro}") == 1 { steps = 1; }
        if nn >= 4 {
            steps = round(nums[nn - 1]);
            obs = take(nums, 0, nn - 1);
        } else {
            if nn == 1 {
                if has(low, "{smotri}") == 0 {
                    steps = round(nums[0]);
                    obs = last;
                    if len(obs) < 1 {
                        println("first {laquo}{smotri} 5 1 6{raquo}, then {laquo}{chto_budet}{hellip}{raquo}");
                        return 1;
                    }
                } else {
                    obs = last;
                }
            } else {
                if nn >= 3 {
                    obs = nums;
                } else {
                    obs = last;
                    if len(obs) < 1 {
                        println("usage: future n | future a b c n");
                        return 1;
                    }
                }
            }
        }
        if steps < 1 { steps = 1; }
        let traj = nt_foresee_n(m, obs, steps);
        println("if start " + fmt_round(obs) + ", then after " + to_str(steps) + " steps:");
        let i: i64 = 0;
        while i < len(traj) {
            println("  +" + to_str(i + 1) + " {arrow} " + fmt_round(traj[i]));
            i = i + 1;
        }
        st[1] = obs;
        return 1;
    }
    if has_any(low, ["see", "{smotri}", "{nablyud}", "{seychas}", "{sostoyanie}", "sense"]) == 1 {
        if nn < 1 {
            print_help();
            return 1;
        }
        let pred = nt_predict(m, nums);
        let fore = nt_foresee(m, nums);
        let s: f64 = nt_surprise(pred, nums);
        println("{seychas}     " + fmt_round(nums));
        println("expect      " + fmt_round(pred));
        println("hybrid      " + fmt_round(fore));
        if s > 0.5 {
            println("this is odd for me (surprise=" + to_str(s) + ")");
        } else {
            println("looks familiar (surprise=" + to_str(s) + ")");
        }
        st[1] = nums;
        return 1;
    }
    if is_num_line(line) == 1 {
        if nn < 1 {
            print_help();
            return 1;
        }
        let pred2 = nt_predict(m, nums);
        let fore2 = nt_foresee(m, nums);
        let s2: f64 = nt_surprise(pred2, nums);
        println("{seychas}     " + fmt_round(nums));
        println("expect      " + fmt_round(pred2));
        println("hybrid      " + fmt_round(fore2));
        if s2 > 0.5 {
            println("this is odd for me (surprise=" + to_str(s2) + ")");
        } else {
            println("looks familiar (surprise=" + to_str(s2) + ")");
        }
        st[1] = nums;
        return 1;
    }
    if has_any(low, ["teach", "{nauchi}", "{zapomni}", "{vyuchi}"]) == 1 {
        if nn < 2 {
            println("usage: {nauchi} 5 1 6 -> 6 1 5");
            return 1;
        }
        let mid: i64 = nn / 2;
        let a = take(nums, 0, mid);
        let b = take(nums, mid, nn);
        nt_remember_next(m, a, b, 0.6);
        let loss: f64 = nt_learn(m, a, b);
        println("remembered " + fmt_round(a) + " {arrow} " + fmt_round(b) + "  (loss=" + to_str(loss) + ")");
        return 1;
    }
    if has(line, "->") == 1 {
        if nn < 2 {
            println("usage: {nauchi} 5 1 6 -> 6 1 5");
            return 1;
        }
        let mid2: i64 = nn / 2;
        let a2 = take(nums, 0, mid2);
        let b2 = take(nums, mid2, nn);
        nt_remember_next(m, a2, b2, 0.6);
        let loss2: f64 = nt_learn(m, a2, b2);
        println("remembered " + fmt_round(a2) + " {arrow} " + fmt_round(b2) + "  (loss=" + to_str(loss2) + ")");
        return 1;
    }
    if has_any(low, ["recall", "{vspomni}", "{pohozh}"]) == 1 {
        if nn < 1 {
            print_help();
            return 1;
        }
        let hits = nt_recall(m, nums, 3);
        println("nearest traces to " + fmt_round(nums) + ":");
        let hi: i64 = 0;
        while hi < len(hits) {
            println("  [" + to_str(hi) + "] " + fmt_round(hits[hi]));
            hi = hi + 1;
        }
        return 1;
    }
    println("unknown command: {laquo}" + line + "{raquo}");
    println("I am a world-model (dim=" + to_str(m[5]) + ", steps=" + to_str(m[6]) + "), not a free chatbot.");
    println("try {laquo}{chto_umeesh}{raquo}, {laquo}{pogovorim}{raquo}, {laquo}{chestno}{raquo} or {laquo}{pomoshch}{raquo}.");
    return 1;
}

fn run_script(st: list, body: str) -> i64 {
    let i: i64 = 0;
    while i < len(body) {
        let line: str = "";
        while i < len(body) {
            let o: i64 = ord(body[i]);
            if o == 10 { break; }
            if o == 13 { break; }
            line = line + body[i];
            i = i + 1;
        }
        if i < len(body) {
            if ord(body[i]) == 13 {
                i = i + 1;
                if i < len(body) {
                    if ord(body[i]) == 10 { i = i + 1; }
                }
            } else {
                i = i + 1;
            }
        }
        line = trim(line);
        if len(line) < 1 { continue; }
        if line[0] == "#" { continue; }
        println("> " + line);
        if handle(st, line) == 0 { return 0; }
    }
    return 0;
}

fn main() -> i64 {
    let path: str = "minds/agent.km";
    if argc() >= 2 {
        if arg(1) != "" { path = arg(1); }
    }
    let script: str = "";
    if argc() >= 3 { script = arg(2); }
    let m = open_mind(path);
    let st: list = [m, [], path];
    println("Kenga chat -- living world-model (native_ml, not C Prophet)");
    println("talk: {laquo}{privet}{raquo}, {laquo}{ty_kto}{raquo}, {laquo}{chto_umeesh}{raquo}, {laquo}{pogovorim}{raquo}, {laquo}{chestno}{raquo}");
    println("model: {laquo}{smotri} 5 1 6{raquo}, {laquo}{chto_budet} {cherez} 4{raquo}, {laquo}{obuchi}{raquo}, {laquo}{status}{raquo}");
    println("");
    if script != "" {
        if file_exists(script) == 0 {
            println("cannot read script " + script);
            return 1;
        }
        return run_script(st, read_file(script));
    }
    while true {
        print("kenga> ");
        let rl = read_line();
        if rl[0] == 0 { break; }
        let line: str = trim(rl[1]);
        if len(line) < 1 { continue; }
        if handle(st, line) == 0 { break; }
    }
    return 0;
}
'''

out = src
for k, v in W.items():
    out = out.replace("{" + k + "}", v)

dst = Path(__file__).with_name("chat.kenga")
dst.write_text(out, encoding="utf-8", newline="\n")
print("wrote", dst, "bytes", dst.stat().st_size)
