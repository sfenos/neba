use std::env;
use std::fs;
use std::process;
use neba_parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_repl(),
        2 => match fs::read_to_string(&args[1]) {
            Ok(source) => run_source(&args[1], &source),
            Err(e) => { eprintln!("neba: cannot read '{}': {}", args[1], e); process::exit(1); }
        },
        _ => { eprintln!("Usage: neba [script.neba]"); process::exit(1); }
    }
}

fn run_source(path: &str, source: &str) {
    println!("── Neba v0.1.1 — parsing: {} ──", path);
    let (program, lex_errors, parse_errors) = parse(source);
    let has_errors = !lex_errors.is_empty() || !parse_errors.is_empty();
    for e in &lex_errors   { eprintln!("  {}", e); }
    for e in &parse_errors { eprintln!("  {}", e); }
    if has_errors { eprintln!("{} error(s).", lex_errors.len() + parse_errors.len()); process::exit(1); }
    println!("  Parsed {} top-level statement(s) — OK", program.stmts.len());
    for (i, stmt) in program.stmts.iter().enumerate() {
        println!("  [{:>2}] line {:>3} — {}", i + 1, stmt.span.line, stmt_label(&stmt.inner));
    }
}

fn stmt_label(s: &neba_parser::StmtKind) -> &'static str {
    use neba_parser::StmtKind;
    match s {
        StmtKind::Let{..}=>"let", StmtKind::Var{..}=>"var", StmtKind::Assign{..}=>"assign",
        StmtKind::Fn{..}=>"fn",   StmtKind::Class{..}=>"class", StmtKind::Trait{..}=>"trait",
        StmtKind::Impl{..}=>"impl",StmtKind::While{..}=>"while",StmtKind::For{..}=>"for",
        StmtKind::Return(_)=>"return", StmtKind::Break=>"break", StmtKind::Continue=>"continue",
        StmtKind::Pass=>"pass", StmtKind::Mod(_)=>"mod", StmtKind::Use(_)=>"use", StmtKind::Expr(_)=>"expr",
    }
}

fn run_repl() {
    use std::io::{self, BufRead, Write};
    println!("Neba REPL v0.1.1 — Ctrl-D to exit");
    loop {
        print!(">>> "); io::stdout().flush().unwrap();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let (program, lex_errors, parse_errors) = parse(&line);
                for e in &lex_errors   { eprintln!("  Lex:   {}", e); }
                for e in &parse_errors { eprintln!("  Parse: {}", e); }
                for stmt in &program.stmts { println!("  {:?}", stmt.inner); }
            }
            Err(e) => { eprintln!("Error: {}", e); break; }
        }
    }
}
