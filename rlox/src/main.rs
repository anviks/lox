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

use crate::{
    interpreter::Interpreter,
    lexer::Lexer,
    parser::Parser,
    resolver::Resolver,
    token::{Span, dump_tokens},
};
use clap::{Parser as ClapParser, ValueEnum};
use owo_colors::{OwoColorize, Style};
use std::{
    cmp::{max, min},
    fs,
    io::IsTerminal,
    process::exit,
};

#[derive(Clone, Debug, ValueEnum)]
enum DumpOption {
    Tokens,
    Ast,
}

#[derive(ClapParser, Debug)]
struct Args {
    filename: String,
    #[arg(long, value_enum)]
    dump: Option<DumpOption>,
}

fn display_error(file_contents: &String, span: &Span, message: &String) {
    let color = std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let styled = |s: Style| if color { s } else { Style::new() };

    let err_style = styled(Style::new().bright_red());
    let err_bold_style = styled(Style::new().bright_red().bold());

    let digits = |n: u32| n.checked_ilog10().unwrap_or(0) as usize + 1;
    let lines: Vec<&str> = file_contents.split('\n').collect();

    let first = max(span.line_start.saturating_sub(2), 1);
    let last = min(span.line_end + 2, lines.len() as u32);
    let width = digits(last);

    eprintln!();
    for i in first..=last {
        let pad = " ".repeat(width.saturating_sub(digits(i)));
        let line = lines[(i - 1) as usize];

        if i != span.line_start {
            eprintln!("{}{} | {}", pad, i, line);
        } else {
            let start = span.col_start as usize;
            let end = span.col_end as usize;

            eprintln!(
                "{}{} | {}{}{}",
                pad,
                i,
                &line[..start - 1],
                (&line[start - 1..end - 1]).style(err_bold_style),
                &line[end - 1..],
            );

            eprintln!(
                "{} |{}{}",
                " ".repeat(width),
                " ".repeat(start),
                "^".repeat(end - start).style(err_bold_style),
            );
        }
    }
    eprintln!("\n{}", message.style(err_style));
}

fn main() {
    let args = Args::parse();
    let filename = &args.filename;

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    });

    let mut lexer = Lexer::new(&file_contents);
    let tokens_result = lexer.analyze();

    if let Err(err) = tokens_result {
        display_error(&file_contents, &err.span, &err.message);
        exit(65);
    }

    let tokens = tokens_result.unwrap();

    if let Some(DumpOption::Tokens) = args.dump {
        println!("{}", dump_tokens(&tokens));
        return;
    }

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_stmts();

    if let Some(DumpOption::Ast) = args.dump
        && let Ok(statements) = &stmts
    {
        for stmt in statements {
            println!("{:#?}", stmt);
        }
        return;
    }

    if let Err(err) = stmts {
        display_error(&file_contents, &err.token.span, &err.message);
        exit(65);
    }

    let statements = stmts.unwrap();

    let mut resolver = Resolver::new();
    if let Err(err) = resolver.resolve_statements(&statements) {
        display_error(&file_contents, &err.token.span, &err.message);
        exit(65);
    }

    let mut interpreter = Interpreter::new();
    interpreter.locals = resolver.locals;
    if let Err(e) = interpreter.interpret(&statements) {
        eprintln!("{}", e.message);
        exit(70);
    }
}
