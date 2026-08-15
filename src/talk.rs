//! Interactive Prophet session + natural-language chat layer.

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::error::{KengaError, Result};
use crate::memory::{
    consolidate, foresee, learn, load_mind, new_memory_config, predict, recall, remember_pair,
    save_mind, surprise_score, train_physics_epoch, unroll, MemoryHandle,
};

pub fn run_talk(mind_path: Option<PathBuf>, scripted: Option<&str>) -> Result<()> {
    let path = mind_path.unwrap_or_else(|| PathBuf::from("minds/agent.km"));
    let mind: MemoryHandle = if path.exists() {
        println!("loading {}", path.display());
        load_mind(&path)?
    } else {
        println!("new mind → will save to {}", path.display());
        new_memory_config(0.05, 128, 48)?
    };

    println!("Kenga chat — живой world-model (не LLM)");
    println!("разговор: «привет», «ты кто», «что умеешь», «поговорим», «честно»");
    println!("модель: «смотри 5 1 6», «что будет через 4», «обучи», «статус»");
    println!();

    if let Some(script) = scripted {
        for line in script.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            println!("> {line}");
            if !dispatch(&mind, &path, line)? {
                break;
            }
        }
        return Ok(());
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("kenga> ");
        stdout.flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !dispatch(&mind, &path, line)? {
            break;
        }
    }
    Ok(())
}

fn dispatch(mind: &MemoryHandle, path: &Path, line: &str) -> Result<bool> {
    let cmd = normalize_line(line);
    handle_line(mind, path, &cmd)
}

fn normalize_line(line: &str) -> String {
    let lower = line.to_lowercase();
    let trimmed = lower.trim();
    let nums = extract_numbers(line);

    if matches_any(&lower, &["quit", "exit", "выход", "пока"]) {
        return "quit".into();
    }
    if matches_any(&lower, &["привет", "hello", "hi", "здравств", "hey"]) {
        return "greet".into();
    }
    // who / what / capabilities / free talk — not command grammar
    // caps/talk BEFORE about: «что ты умеешь» contains «что ты»
    if matches_any(
        &lower,
        &[
            "что умеешь",
            "что ты умеешь",
            "что можешь",
            "возможности",
            "capabilities",
            "что умеет",
            "на что способен",
        ],
    ) {
        return "caps".into();
    }
    if matches_any(
        &lower,
        &[
            "поговори",
            "поговорим",
            "давай поговорим",
            "просто поговорить",
            "хочу поговорить",
            "узнать что ты",
            "давай болтать",
            "chat with me",
            "let's talk",
        ],
    ) {
        return "talk".into();
    }
    if matches_any(
        &lower,
        &[
            "ты кто",
            "кто ты",
            "расскажи о себе",
            "о себе",
            "what are you",
            "who are you",
        ],
    ) || trimmed == "что ты"
        || trimmed.starts_with("что ты такое")
    {
        return "about".into();
    }
    if matches_any(
        &lower,
        &[
            "честно",
            "ты ии",
            "ты llm",
            "как grok",
            "как gpt",
            "как chatgpt",
            "как человек",
            "нейросеть",
            "языковая модель",
        ],
    ) {
        return "honest".into();
    }
    // "?" alone / help words — NOT every question mark in a sentence
    if trimmed == "?"
        || trimmed == "help"
        || trimmed == "помощь"
        || trimmed.starts_with("help ")
        || trimmed.starts_with("помощь")
    {
        return "help".into();
    }
    if matches_any(&lower, &["status", "статус", "как дела", "stats"]) {
        return "status".into();
    }
    if matches_any(&lower, &["sleep", "спи", "консолид", "засни"]) {
        return "sleep".into();
    }
    if matches_any(&lower, &["save", "сохрани", "запиши mind"]) {
        return "save".into();
    }
    if matches_any(&lower, &["load", "загрузи"]) {
        return "load".into();
    }

    if matches_any(&lower, &["train", "обучи", "тренир", "учись"]) {
        let n = nums.first().copied().unwrap_or(8.0) as i64;
        return format!("train {n}");
    }

    // «завтра» / «скоро» ≈ 1 шаг; «через N» берём из чисел
    let futureish = matches_any(
        &lower,
        &[
            "future",
            "будущ",
            "через",
            "что будет",
            "разверн",
            "unroll",
            "предскажи",
            "завтра",
            "скоро",
            "дальше",
        ],
    );
    if futureish {
        let default_steps = if lower.contains("завтра") || lower.contains("скоро") {
            1
        } else {
            4
        };
        if nums.len() >= 4 {
            let steps = nums[nums.len() - 1] as i64;
            let obs: Vec<String> = nums[..nums.len() - 1]
                .iter()
                .map(|n| format!("{n}"))
                .collect();
            return format!("future {} {steps}", obs.join(" "));
        }
        if nums.len() == 1 && !lower.contains("смотри") && !lower.contains("сейчас") {
            // «через 4» or lone step count
            return format!("future {}", nums[0] as i64);
        }
        if nums.len() >= 3 {
            let obs: Vec<String> = nums.iter().map(|n| format!("{n}")).collect();
            return format!("future {} {default_steps}", obs.join(" "));
        }
        return format!("future {default_steps}");
    }

    if matches_any(
        &lower,
        &["see", "смотри", "наблюд", "сейчас", "состояние", "sense"],
    ) {
        if nums.is_empty() {
            return "help".into();
        }
        let obs: Vec<String> = nums.iter().map(|n| format!("{n}")).collect();
        return format!("see {}", obs.join(" "));
    }

    if matches_any(&lower, &["teach", "научи", "запомни", "выучи"]) {
        if let Some(rewritten) = rewrite_teach(line, &nums) {
            return rewritten;
        }
    }

    if matches_any(&lower, &["recall", "вспомни", "похож"]) {
        if nums.is_empty() {
            return "help".into();
        }
        let obs: Vec<String> = nums.iter().map(|n| format!("{n}")).collect();
        return format!("recall {}", obs.join(" "));
    }

    // bare numbers → see
    if !nums.is_empty()
        && line
            .split_whitespace()
            .all(|t| t.parse::<f64>().is_ok() || t == "->" || t == "→")
    {
        if line.contains("->") || line.contains('→') {
            if let Some(r) = rewrite_teach(line, &nums) {
                return r;
            }
        }
        let obs: Vec<String> = nums.iter().map(|n| format!("{n}")).collect();
        return format!("see {}", obs.join(" "));
    }

    line.to_string()
}

fn rewrite_teach(line: &str, nums: &[f64]) -> Option<String> {
    if nums.len() < 2 {
        return None;
    }
    let mid = nums.len() / 2;
    if line.contains("->") || line.contains('→') || line.contains("станет") || line.contains("->")
    {
        let left: Vec<String> = nums[..mid].iter().map(|n| format!("{n}")).collect();
        let right: Vec<String> = nums[mid..].iter().map(|n| format!("{n}")).collect();
        return Some(format!("teach {} -> {}", left.join(" "), right.join(" ")));
    }
    if nums.len() >= 6 {
        let left: Vec<String> = nums[..3].iter().map(|n| format!("{n}")).collect();
        let right: Vec<String> = nums[3..6].iter().map(|n| format!("{n}")).collect();
        return Some(format!("teach {} -> {}", left.join(" "), right.join(" ")));
    }
    None
}

fn matches_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

fn extract_numbers(s: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() || ch == '.' || (ch == '-' && cur.is_empty()) {
            cur.push(ch);
        } else if !cur.is_empty() {
            if let Ok(n) = cur.parse() {
                out.push(n);
            }
            cur.clear();
        }
    }
    if !cur.is_empty() {
        if let Ok(n) = cur.parse() {
            out.push(n);
        }
    }
    out
}

fn handle_line(mind: &MemoryHandle, path: &Path, line: &str) -> Result<bool> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(true);
    }
    match parts[0] {
        "quit" | "exit" | "q" => {
            println!("пока — mind на диске, если сохранил");
            return Ok(false);
        }
        "greet" => {
            let m = mind.borrow();
            println!(
                "привет. я живой world-model Kenga (steps={}, dim={}).",
                m.model.steps, m.model.dim
            );
            if m.model.dim >= 9 {
                println!("этот mind — multimodal: картинка+звук → вектор → предсказание.");
            } else {
                println!("этот mind — физика агента [pos vel fuel].");
            }
            println!("можешь: «что умеешь», «поговорим», «смотри …», «помощь».");
        }
        "about" => {
            let m = mind.borrow();
            println!(
                "я Prophet-mind на языке Kenga: residual MLP, dim={}, hidden={}, steps={}.",
                m.model.dim, m.model.hidden, m.model.steps
            );
            println!(
                "episodic={}, core={}. учусь переходам obs→next, удивляюсь, сплю (consolidate).",
                m.episodic.len(),
                m.core.len()
            );
            if m.model.dim >= 9 {
                println!("обучен на PPM+WAV сценах (living multimodal).");
            }
            println!("я ещё не чат-LLM вроде Grok — но я живу в цикле sense→learn→sleep.");
        }
        "caps" => {
            let m = mind.borrow();
            println!("умею сейчас:");
            println!("  • предсказывать следующий вектор состояния (predict / unroll)");
            println!("  • удивляться странному (surprise) и помнить эпизоды");
            println!("  • спать: consolidate → ядро с EWC-lite");
            println!("  • сохраняться в .km и подниматься в chat");
            if m.model.dim >= 9 {
                println!("  • multimodal obs: RGB + audio energy + time (dim=9)");
                println!("    демо: kenga run examples/ml/living_multimodal.kenga");
            } else {
                println!("  • физика [pos, vel, fuel] — «смотри 5 1 6»");
            }
            println!("пока не умею: свободный диалог как у большой LLM (это следующий слой — tiny LM).");
            println!("команды: статус | обучи | спи | сохрани | помощь");
        }
        "talk" => {
            let m = mind.borrow();
            println!("давай. я не болтаю про всё на свете — я говорю о том, что помню телом модели.");
            println!(
                "сейчас во мне: {} шагов обучения, dim={}, core={}.",
                m.model.steps, m.model.dim, m.core.len()
            );
            if m.model.dim >= 9 {
                println!("спроси «что умеешь» или дай кадр цифрами / переобучи multimodal demo.");
            } else {
                println!("скажи «смотри 5 1 6» — покажу, что жду дальше. или «что будет через 4».");
            }
            println!("если нужна честность про потолок — напиши «честно».");
        }
        "honest" => {
            println!("честно: сейчас я world-model (векторы → векторы), не языковая модель.");
            println!("«умная как Grok» = отдельный путь: tiny LM → больше данных → f32/GPU.");
            println!("живой я уже: учусь, удивляюсь, сплю, пишусь на диск. это Kenga.");
            println!("демо языка: kenga run examples/ml/tiny_lm.kenga");
        }
        "help" | "?" => {
            println!("разговор:  привет | ты кто | что умеешь | поговорим | честно");
            println!("модель:    смотри 5 1 6 | что будет через 4 | обучи 10");
            println!("память:    научи 5 1 6 станет 6 1 5 | вспомни … | спи | сохрани");
            println!("выход:     выход / quit");
        }
        "status" => {
            let m = mind.borrow();
            println!(
                "я помню: ep={} core={} steps={} dim={}x{} (lr={})",
                m.episodic.len(),
                m.core.len(),
                m.model.steps,
                m.model.dim,
                m.model.hidden,
                m.lr
            );
        }
        "train" => {
            let n = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(8usize);
            println!("учусь {n} эпох…");
            for e in 0..n {
                let loss = train_physics_epoch(&mut mind.borrow_mut());
                println!("  эпоха {e}: loss={loss:.6}");
            }
            let folded = consolidate(&mut mind.borrow_mut());
            println!("поспал, сложил эпизодов: {folded}");
        }
        "see" => {
            let obs = parse_vec(&parts[1..])?;
            if obs.is_empty() {
                return Err(KengaError::new("usage: see <nums...>", None));
            }
            let m = mind.borrow();
            let pred = predict(&m, &obs);
            let fore = foresee(&m, &obs);
            let s = surprise_score(&pred, &obs);
            println!("сейчас     {}", fmt_round(&obs));
            println!("я жду      {}", fmt_round(&pred));
            println!("(точно)    {}", fmt_vec(&pred));
            println!("гибрид     {}", fmt_round(&fore));
            if s > 0.5 {
                println!("это для меня странновато (surprise={s:.3})");
            } else {
                println!("похоже на знакомое (surprise={s:.3})");
            }
            drop(m);
            LAST_OBS.with(|c| *c.borrow_mut() = obs);
        }
        "future" => {
            let (obs, steps) = if parts.len() == 2
                && parts[1]
                    .chars()
                    .all(|c| c == '-' || c.is_ascii_digit())
            {
                let steps: usize = parts[1].parse().unwrap_or(4);
                let obs = LAST_OBS.with(|c| c.borrow().clone());
                if obs.is_empty() {
                    return Err(KengaError::new(
                        "сначала скажи состояние: «смотри 5 1 6», потом «что будет завтра?»",
                        None,
                    ));
                }
                (obs, steps)
            } else if parts.len() >= 3 {
                let steps: usize = parts[parts.len() - 1].parse().unwrap_or(4);
                let obs = parse_vec(&parts[1..parts.len() - 1])?;
                (obs, steps)
            } else {
                return Err(KengaError::new("usage: future n | future a b c n", None));
            };
            let traj = unroll(&mind.borrow(), &obs, steps);
            println!("если старт {}, то через {steps} шагов:", fmt_round(&obs));
            for (i, s) in traj.iter().enumerate() {
                println!("  +{} → {}", i + 1, fmt_round(s));
            }
            LAST_OBS.with(|c| *c.borrow_mut() = obs);
        }
        "teach" => {
            let arrow = parts.iter().position(|p| *p == "->" || *p == "→");
            let Some(ai) = arrow else {
                return Err(KengaError::new("usage: teach a b c -> x y z", None));
            };
            let x = parse_vec(&parts[1..ai])?;
            let y = parse_vec(&parts[ai + 1..])?;
            remember_pair(&mut mind.borrow_mut(), x.clone(), Some(y.clone()), 0.6, 0);
            let loss = learn(&mut mind.borrow_mut(), &x, &y);
            println!(
                "запомнил {} → {}  (loss={loss:.4})",
                fmt_round(&x),
                fmt_round(&y)
            );
        }
        "sleep" => {
            let n = consolidate(&mut mind.borrow_mut());
            println!("сон: в ядро ушло {n}");
        }
        "recall" => {
            let q = parse_vec(&parts[1..])?;
            let hits = recall(&mind.borrow(), &q, 3);
            println!("ближайшие следы к {}:", fmt_round(&q));
            for (i, h) in hits.iter().enumerate() {
                println!("  [{i}] {}", fmt_round(h));
            }
        }
        "save" => {
            let p = parts
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(|| path.to_path_buf());
            save_mind(&mind.borrow(), &p)?;
            println!("сохранил {}", p.display());
        }
        "load" => {
            let p = parts
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(|| path.to_path_buf());
            let loaded = load_mind(&p)?;
            *mind.borrow_mut() = loaded.borrow().clone();
            println!("загрузил {}", p.display());
        }
        other => {
            // parts[0] alone was misleading («я» from «я хочу…»)
            let full = line.trim();
            let m = mind.borrow();
            println!("не распознал как команду: «{full}»");
            println!(
                "я world-model (dim={}, steps={}), не свободный чат-бот.",
                m.model.dim, m.model.steps
            );
            println!("попробуй «что умеешь», «поговорим», «честно» или «помощь».");
            let _ = other;
        }
    }
    Ok(true)
}

fn parse_vec(parts: &[&str]) -> Result<Vec<f64>> {
    parts
        .iter()
        .map(|t| {
            t.parse::<f64>()
                .map_err(|_| KengaError::new(format!("not a number: {t}"), None))
        })
        .collect()
}

fn fmt_vec(xs: &[f64]) -> String {
    xs.iter()
        .map(|x| format!("{x:.4}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn fmt_round(xs: &[f64]) -> String {
    xs.iter()
        .map(|x| format!("{}", x.round() as i64))
        .collect::<Vec<_>>()
        .join(" ")
}

thread_local! {
    static LAST_OBS: std::cell::RefCell<Vec<f64>> = std::cell::RefCell::new(Vec::new());
}
