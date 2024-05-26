#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    repl_start();
}

fn repl_start() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        prompt_print(&mut stdout);
        let input = input_read(&stdin);
        match command_parse(&input) {
            Ok(output) => println!("{output}"),
            Err(e) => println!("{e}"),
        }
    }
}

fn command_parse(input: &str) -> Result<String, String> {
    let mut input_token_iter = input.split_whitespace();
    match input_token_iter.next() {
        Some("exit") => match input_token_iter.next() {
            Some(arg_str) => match arg_str.parse::<i32>() {
                Ok(code) => process::exit(code),
                _ => process::exit(1),
            },
            _ => process::exit(0),
        },
        Some(cmd @ "echo") => Ok(input
            .chars()
            .skip(cmd.len())
            .skip_while(|c| c.is_whitespace())
            .collect()),
        Some(cmd) => Err(format!("{cmd}: command not found")),
        _ => Ok(String::new()),
    }
}

fn input_read(stdin: &io::Stdin) -> String {
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    input.trim().to_owned()
}

fn prompt_print(stdout: &mut io::Stdout) {
    print!("$ ");
    stdout.flush().unwrap();
}
