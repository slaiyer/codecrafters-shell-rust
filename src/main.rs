#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;
use strum::{AsRefStr, EnumString};
use thiserror::Error;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    repl_start();
}

fn repl_start() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        print!("$ ");
        stdout.flush().unwrap();
        stderr.flush().unwrap();
        let input = input_read(&stdin);
        let cmd = match input.split_ascii_whitespace().next() {
            Some(cmd) => cmd,
            _ => {
                println!();
                continue;
            }
        };
        let args = input
            .chars()
            .skip(cmd.len())
            .collect::<String>()
            .trim()
            .to_owned();
        match cmd.parse::<Command>() {
            Ok(command) => match command.build(&args) {
                Ok(command) => command.execute(),
                Err(e) => eprintln!("{e}"),
            },
            Err(_) => eprintln!("{cmd}: command not found"),
        }
    }
}

#[derive(Debug, PartialEq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
enum Command {
    Exit { code: i32 },
    Echo { message: String },
    Type { tokens: Vec<String> },
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("invalid arguments: {0}")]
    Argument(String),
}

impl Command {
    fn build(self, args: &str) -> Result<Self, CommandError> {
        let tokens = args
            .split_ascii_whitespace()
            .map(str::to_string)
            .collect::<Vec<String>>();
        match self {
            Self::Exit { .. } => match tokens.len() {
                n if n > 1 => Err(CommandError::Argument("too many supplied".to_owned())),
                1 => Ok(Self::Exit {
                    code: match tokens[0].parse::<i32>() {
                        Ok(code) => code,
                        _ => 1,
                    },
                }),
                _ => Ok(Self::Exit { code: 0 }),
            },
            Self::Echo { .. } => Ok(Self::Echo {
                message: args.to_owned(),
            }),
            Self::Type { .. } => Ok(Self::Type { tokens }),
        }
    }

    fn execute(self) {
        match self {
            Self::Exit { code } => process::exit(code),
            Self::Echo { message } => println!("{message}"),
            Self::Type { tokens } => {
                for t in tokens {
                    match t.parse::<Command>() {
                        Ok(t) => println!("{} is a shell builtin", t.as_ref()),
                        Err(_) => println!("{t} not found"),
                    }
                }
            }
        }
    }
}

fn input_read(stdin: &io::Stdin) -> String {
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    input.trim().to_owned()
}
