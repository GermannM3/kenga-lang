//! `kenga build`: emit-c then invoke a system C compiler.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::codegen::emit_c;
use crate::driver::parse_file;
use crate::error::{KengaError, Result};

pub struct BuildOptions {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub keep_c: bool,
}

pub fn build(opts: BuildOptions) -> Result<PathBuf> {
    let program = parse_file(&opts.input)?;
    let c_src = emit_c(&program)?;

    let stem = opts
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("kenga_out");
    let c_path = opts
        .input
        .with_file_name(format!("{stem}.kenga.c"));
    fs::write(&c_path, &c_src).map_err(|e| {
        KengaError::new(format!("cannot write {}: {e}", c_path.display()), None)
    })?;

    let out = opts.output.unwrap_or_else(|| {
        if cfg!(windows) {
            opts.input.with_file_name(format!("{stem}.exe"))
        } else {
            opts.input.with_file_name(stem)
        }
    });

    let cc = find_cc()?;
    let status = compile_with(&cc, &c_path, &out)?;
    if !status {
        return Err(KengaError::new(
            format!(
                "C compiler failed ({cc}). Source left at {}",
                c_path.display()
            ),
            None,
        ));
    }

    if !opts.keep_c {
        let _ = fs::remove_file(&c_path);
    }
    Ok(out)
}

fn find_cc() -> Result<String> {
    for cand in ["gcc", "clang", "cc", "cl"] {
        if Command::new(cand)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || (cand == "cl"
                && Command::new(cand)
                    .output()
                    .map(|o| o.status.code().is_some())
                    .unwrap_or(false))
        {
            return Ok(cand.into());
        }
    }
    if which("gcc") {
        return Ok("gcc".into());
    }
    if which("clang") {
        return Ok("clang".into());
    }
    if which("cl") {
        return Ok("cl".into());
    }
    // Windows: MSVC via vswhere / BuildTools (cl often not on PATH)
    if let Some(cl) = find_msvc_cl() {
        return Ok(cl);
    }
    // LLVM clang with absolute path (may still need CRT)
    let llvm = Path::new(r"C:\Program Files\LLVM\bin\clang.exe");
    if llvm.is_file() {
        return Ok(llvm.display().to_string());
    }
    Err(KengaError::new(
        "no C compiler found (install gcc, clang, or MSVC Build Tools)",
        None,
    ))
}

fn find_msvc_cl() -> Option<String> {
    let vswhere = Path::new(
        r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
    );
    if vswhere.is_file() {
        let out = Command::new(vswhere)
            .args([
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-find",
                r"VC\Tools\MSVC\*\bin\Hostx64\x64\cl.exe",
            ])
            .output()
            .ok()?;
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = text.lines().next() {
                let p = line.trim();
                if !p.is_empty() && Path::new(p).is_file() {
                    return Some(p.to_string());
                }
            }
        }
    }
    None
}

fn which(name: &str) -> bool {
    Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn compile_with(cc: &str, c_path: &Path, out: &Path) -> Result<bool> {
    let output = if cc == "cl" {
        Command::new(cc)
            .arg("/nologo")
            .arg(c_path)
            .arg(format!("/Fe:{}", out.display()))
            .output()
    } else {
        Command::new(cc)
            .arg(c_path)
            .arg("-O2")
            .arg("-o")
            .arg(out)
            .output()
    }
    .map_err(|e| KengaError::new(format!("failed to spawn {cc}: {e}"), None))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("{stdout}{stderr}");
        return Ok(false);
    }
    Ok(true)
}
