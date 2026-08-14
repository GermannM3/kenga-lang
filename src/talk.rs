//! Interactive Prophet session: train, sense, foresee, teach, save.

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

    println!("Kenga talk — живой world-model (не чат-LLM)");
    println!("команды: train | see a b c | future n | teach a b c -> x y z | sleep | recall a b c");
    println!("         status | save | load | help | quit");
    println!();

    if let Some(script) = scripted {
        for line in script.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            println!("> {line}");
            if !handle_line(&mind, &path, line)? {
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
        if !handle_line(&mind, &path, line)? {
            break;
        }
    }
    Ok(())
}

fn handle_line(mind: &MemoryHandle, path: &Path, line: &str) -> Result<bool> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(true);
    }
    match parts[0] {
        "quit" | "exit" | "q" => {
            println!("bye");
            return Ok(false);
        }
        "help" | "?" => {
            println!("train          — эпоха физики агента");
            println!("see 5 1 6      — наблюдение → predict + foresee + surprise");
            println!("future 4       — unroll от последнего see / или future 5 1 6 4");
            println!("teach 5 1 6 -> 6 1 5");
            println!("sleep          — consolidate");
            println!("recall 5 1 6   — ближайшие следы");
            println!("status | save | load");
        }
        "status" => {
            let m = mind.borrow();
            println!(
                "ep={} core={} steps={} dim={} hidden={} lr={}",
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
            for e in 0..n {
                let loss = train_physics_epoch(&mut mind.borrow_mut());
                println!("epoch {e}: loss={loss:.6}");
            }
            let folded = consolidate(&mut mind.borrow_mut());
            println!("sleep folded={folded}");
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
            println!("obs     {}", fmt_vec(&obs));
            println!("predict {}", fmt_vec(&pred));
            println!("round   {}", fmt_round(&pred));
            println!("foresee {}", fmt_vec(&fore));
            println!("surprise_vs_obs {s:.4}");
            drop(m);
            // stash last obs in a side channel via remember of last_see? keep simple: print only
            LAST_OBS.with(|c| *c.borrow_mut() = obs);
        }
        "future" => {
            let (obs, steps) = if parts.len() >= 2 && parts[1].chars().all(|c| c == '-' || c.is_ascii_digit())
                && parts.len() == 2
            {
                let steps: usize = parts[1].parse().unwrap_or(4);
                let obs = LAST_OBS.with(|c| c.borrow().clone());
                if obs.is_empty() {
                    return Err(KengaError::new(
                        "future n needs prior `see`, or: future a b c n",
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
            println!("unroll from {}", fmt_vec(&obs));
            for (i, s) in traj.iter().enumerate() {
                println!("  t+{}  {}  ({})", i + 1, fmt_round(s), fmt_vec(s));
            }
        }
        "teach" => {
            // teach 5 1 6 -> 6 1 5
            let arrow = parts.iter().position(|p| *p == "->" || *p == "→");
            let Some(ai) = arrow else {
                return Err(KengaError::new("usage: teach a b c -> x y z", None));
            };
            let x = parse_vec(&parts[1..ai])?;
            let y = parse_vec(&parts[ai + 1..])?;
            remember_pair(&mut mind.borrow_mut(), x.clone(), Some(y.clone()), 0.6, 0);
            let loss = learn(&mut mind.borrow_mut(), &x, &y);
            println!("learned {} -> {}  loss={loss:.6}", fmt_vec(&x), fmt_vec(&y));
        }
        "sleep" => {
            let n = consolidate(&mut mind.borrow_mut());
            println!("folded {n}");
        }
        "recall" => {
            let q = parse_vec(&parts[1..])?;
            let hits = recall(&mind.borrow(), &q, 3);
            for (i, h) in hits.iter().enumerate() {
                println!("  [{i}] {}", fmt_vec(h));
            }
        }
        "save" => {
            let p = parts.get(1).map(PathBuf::from).unwrap_or_else(|| path.to_path_buf());
            save_mind(&mind.borrow(), &p)?;
            println!("saved {}", p.display());
        }
        "load" => {
            let p = parts.get(1).map(PathBuf::from).unwrap_or_else(|| path.to_path_buf());
            let loaded = load_mind(&p)?;
            *mind.borrow_mut() = loaded.borrow().clone();
            println!("loaded {}", p.display());
        }
        other => {
            println!("unknown command '{other}' (help)");
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
