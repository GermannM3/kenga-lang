use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{Item, Program};
use crate::bytecode::Module;
use crate::compiler::compile_with_options;
use crate::error::{KengaError, Result};
use crate::lexer::Lexer;
use crate::parser::Parser;

/// Load a .kenga file and recursively merge imports into one Program.
pub fn load_program(path: &Path) -> Result<Program> {
    let mut visiting = HashSet::new();
    load_recursive(path, &mut visiting)
}

fn load_recursive(path: &Path, visiting: &mut HashSet<PathBuf>) -> Result<Program> {
    let canon = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    if !visiting.insert(canon.clone()) {
        return Err(KengaError::new(
            format!("circular import: {}", path.display()),
            None,
        ));
    }

    let src = fs::read_to_string(path).map_err(|e| {
        KengaError::new(format!("cannot read {}: {e}", path.display()), None)
    })?;
    let tokens = Lexer::new(&src).tokenize()?;
    let mut program = Parser::new(tokens).parse()?;

    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let imports = program.imports.clone();
    program.imports.clear();

    let mut merged_items: Vec<Item> = Vec::new();
    for imp in imports {
        let imp_path = resolve_import(base, &imp.path)?;
        let dep = load_recursive(&imp_path, visiting)?;
        merged_items.extend(dep.items);
    }
    merged_items.extend(program.items);
    program.items = merged_items;

    visiting.remove(&canon);
    Ok(program)
}

fn resolve_import(base: &Path, spec: &str) -> Result<PathBuf> {
    let candidate = base.join(spec);
    if candidate.exists() {
        return Ok(candidate);
    }
    // try from cwd / repo root style
    let cwd = PathBuf::from(spec);
    if cwd.exists() {
        return Ok(cwd);
    }
    Err(KengaError::new(
        format!("import not found: \"{spec}\" (from {})", base.display()),
        None,
    ))
}

pub fn compile_file(path: &Path) -> Result<Module> {
    let program = load_program(path)?;
    compile_with_options(&program, true)
}

pub fn parse_file(path: &Path) -> Result<Program> {
    load_program(path)
}
