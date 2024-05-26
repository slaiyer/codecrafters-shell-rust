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
        if let Err(e) = command_parse(&input) {
            println!("{e}")
        }
    }
}

fn command_parse(input: &str) -> Result<String, String> {
    match input {
        "exit 0" => process::exit(0),
        _ => Err(format!("{input}: command not found")),
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
