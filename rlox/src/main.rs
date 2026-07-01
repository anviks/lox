mod ast;
mod environment;
mod helpers;
mod interpreter;
mod lexer;
mod natives;
mod parser;
mod resolver;
mod token;
mod value;

use std::{env, fs, process::exit};

use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser, resolver::Resolver};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut lexer = Lexer::new(&file_contents);
            let tokens = lexer.analyze();

            for tok in tokens {
                println!("{}", tok)
            }

            if lexer.encountered_error {
                exit(65);
            }
        }
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut lexer = Lexer::new(&file_contents);
            let tokens = lexer.analyze();

            if lexer.encountered_error {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            let expr = parser.parse();
            println!(
                "{}",
                match expr {
                    Some(ex) => ex.to_string(),
                    None => String::new(),
                }
            );

            if parser.encountered_error {
                exit(65);
            }
        }
        "evaluate" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut lexer = Lexer::new(&file_contents);
            let tokens = lexer.analyze();

            if lexer.encountered_error {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            let expr = parser.parse();

            if parser.encountered_error || expr.is_none() {
                exit(65);
            }

            let mut interpreter = Interpreter::new();
            match interpreter.evaluate(&expr.unwrap()) {
                Ok(val) => println!("{}", val.to_string()),
                Err(e) => {
                    eprintln!("{}", e.message);
                    exit(70);
                }
            }
        }
        "run" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut lexer = Lexer::new(&file_contents);
            let tokens = lexer.analyze();

            if lexer.encountered_error {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            let stmts = parser.parse_stmts();

            if parser.encountered_error || stmts.is_err() {
                eprintln!("{}", stmts.unwrap_err().message);
                exit(65);
            }

            let statements = stmts.unwrap();

            let mut resolver = Resolver::new();
            if let Err(err) = resolver.resolve_statements(&statements) {
                let relevant_lines: Vec<&str> = file_contents.split('\n').collect();
                let line_nr_width = ((err.token.span.line_end + 1).ilog10() + 1) as usize;
                eprintln!("\n");
                for i in err.token.span.line_start - 2..=err.token.span.line_end + 2 {
                    if i != err.token.span.line_start {
                        eprintln!(
                            "{}{} | {}",
                            " ".repeat(line_nr_width - (i.ilog10() + 1) as usize),
                            i,
                            relevant_lines[(i - 1) as usize]
                        );
                    } else {
                        let start = err.token.span.col_start as usize;
                        let end = err.token.span.col_end as usize;

                        let line = relevant_lines[(i - 1) as usize];

                        eprintln!(
                            "{}{} | {}{}{}",
                            " ".repeat(line_nr_width - (i.ilog10() + 1) as usize),
                            i,
                            &line[..start - 1],
                            "\x1B[1m\x1B[91m".to_string()
                                + &line[start - 1..end - 1]
                                + "\x1B[39m\x1B[22m",
                            &line[end - 1..],
                        );

                        eprintln!(
                            "{} |{}",
                            " ".repeat(line_nr_width),
                            " ".repeat(start)
                                + "\x1B[1m\x1B[91m"
                                + &"^".repeat(end - start)
                                + "\x1B[39m\x1B[22m",
                        );
                    }
                }
                eprintln!("\n\x1B[91m{}\x1B[39m", err.message);
                exit(65);
            }

            let mut interpreter = Interpreter::new();
            interpreter.locals = resolver.locals;
            if let Err(e) = interpreter.interpret(&statements) {
                eprintln!("{}", e.message);
                exit(70);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
