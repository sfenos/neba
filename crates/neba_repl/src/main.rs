//! neba_repl — REPL interattivo per Neba v0.1.2
//! Supporta: espressioni, statement, definizioni di funzioni/classi multi-riga.
//! Comandi speciali: :quit / :q, :clear, :help

use std::io::{self, BufRead, Write};
use neba_interpreter::{Interpreter, Value};

const BANNER: &str = r#"
  ███╗   ██╗███████╗██████╗  █████╗
  ████╗  ██║██╔════╝██╔══██╗██╔══██╗
  ██╔██╗ ██║█████╗  ██████╔╝███████║
  ██║╚██╗██║██╔══╝  ██╔══██╗██╔══██║
  ██║ ╚████║███████╗██████╔╝██║  ██║
  ╚═╝  ╚═══╝╚══════╝╚═════╝ ╚═╝  ╚═╝  v0.1.2
"#;

fn main() {
    println!("{}", BANNER);
    println!("  Tree-walking interpreter — type :help for commands\n");

    let mut interp = Interpreter::new();
    let mut pending = String::new();

    loop {
        let prompt = if pending.is_empty() { ">>> " } else { "... " };
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) => { println!(); break; }
            Err(e) => { eprintln!("Error: {}", e); break; }
            Ok(_) => {}
        }

        let trimmed = line.trim_end_matches('\n');

        // Comandi speciali
        match trimmed.trim() {
            ":quit" | ":q" | "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            ":clear" => {
                interp = Interpreter::new();
                pending.clear();
                println!("  Environment cleared.");
                continue;
            }
            ":help" => {
                print_help();
                continue;
            }
            _ => {}
        }

        // Accumulo multiriga: se la linea termina con ':' o il blocco è aperto
        pending.push_str(trimmed);
        pending.push('\n');

        // Prova a parsare: se parse_errors = solo MissingIndent/Eof → continua
        let needs_more = should_continue(&pending);
        if needs_more {
            continue;
        }

        // Esegui
        let source = std::mem::take(&mut pending);
        eval_and_print(&mut interp, &source);
    }
}

fn eval_and_print(interp: &mut Interpreter, source: &str) {
    let (program, lex_errors, parse_errors) = neba_parser::parse(source);

    for e in &lex_errors   { eprintln!("  \x1b[31m[Lex]\x1b[0m   {}", e); }
    for e in &parse_errors { eprintln!("  \x1b[31m[Parse]\x1b[0m {}", e); }
    if !lex_errors.is_empty() || !parse_errors.is_empty() { return; }

    for stmt in &program.stmts {
        match interp.exec_stmt(stmt) {
            Ok(Value::None) => {}
            Ok(v) if matches!(v, Value::__Return(_) | Value::__Break | Value::__Continue) => {}
            Ok(v)  => println!("  \x1b[32m{}\x1b[0m", v),
            Err(e) => eprintln!("  \x1b[31m[Runtime]\x1b[0m {}", e),
        }
    }
}

/// Restituisce true se l'input sembra incompleto (blocco aperto).
fn should_continue(src: &str) -> bool {
    use neba_parser::ParseError;
    let (_, _, errs) = neba_parser::parse(src);
    errs.iter().any(|e| matches!(e, ParseError::MissingIndent { .. }))
        // Heuristic: ultima riga non vuota termina con un keyword che apre un blocco
        || src.lines()
              .rev()
              .find(|l| !l.trim().is_empty())
              .map(|l| {
                  let t = l.trim();
                  t.ends_with(':')    // non usato in Neba, ma per sicurezza
                  || matches!(t.split_whitespace().next(), Some("fn") | Some("class") | Some("trait") | Some("if") | Some("while") | Some("for") | Some("match") | Some("async") | Some("elif") | Some("else") | Some("impl"))
              })
              .unwrap_or(false)
}

fn print_help() {
    println!("  Neba REPL v0.1.2 — comandi:");
    println!("  :quit / :q    Esci dal REPL");
    println!("  :clear        Azzera l'environment (variabili/funzioni)");
    println!("  :help         Mostra questo messaggio");
    println!();
    println!("  Inserisci codice Neba direttamente. Per blocchi multi-riga");
    println!("  (fn, class, if, while, for, match) continua su righe successive");
    println!("  indentate con 4 spazi. Una riga vuota chiude il blocco.");
}
