use std::env;
use std::fs;
use std::process;

use neba_parser::parse;
use neba_vm::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_repl(),
        2 => match fs::read_to_string(&args[1]) {
            Ok(source) => run_source(&args[1], &source),
            Err(e) => {
                eprintln!("neba: cannot read '{}': {}", args[1], e);
                process::exit(1);
            }
        },
        _ => {
            eprintln!("Usage: neba [script.neba]");
            process::exit(1);
        }
    }
}

fn run_source(path: &str, source: &str) {
    // 1. Parse
    let (program, lex_errors, parse_errors) = parse(source);
    let has_errors = !lex_errors.is_empty() || !parse_errors.is_empty();
    for e in &lex_errors   { eprintln!("[LexError] {}", e); }
    for e in &parse_errors { eprintln!("[ParseError] {}", e); }
    if has_errors {
        eprintln!("{} error(s).", lex_errors.len() + parse_errors.len());
        process::exit(1);
    }

    // 2. Esegui con la VM
    match run(source) {
        Ok(_)  => {}
        Err(e) => {
            eprintln!("[RuntimeError] {}", e);
            process::exit(1);
        }
    }
}

fn run_repl() {
    use std::io::{self, BufRead, Write};
    println!("Neba REPL v0.2.2 — Ctrl-D to exit");
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                match run(&line) {
                    Ok(v) => {
                        let s = format!("{:?}", v);
                        if s != "None" { println!("{}", s); }
                    }
                    Err(e) => eprintln!("  Error: {}", e),
                }
            }
            Err(e) => { eprintln!("Error: {}", e); break; }
        }
    }
}
