#[allow(unused_imports)]
use std::env;
use std::process;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use std::{
    io::{self, Write},
    path::PathBuf,
};
use strum::{AsRefStr, EnumString};
use thiserror::Error;

fn main() {
    let paths = env::split_paths(&env::var("PATH").unwrap())
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        stderr.flush().unwrap();
        print!("$ ");
        stdout.flush().unwrap();

        let input = input_read(&stdin);
        let cmd = match input.split_ascii_whitespace().next() {
            Some(cmd) => cmd,
            _ => continue,
        };
        let args = input
            .chars()
            .skip(cmd.len())
            .collect::<String>()
            .trim()
            .to_owned();
        match cmd.parse::<Command>() {
            Ok(command) => match command.build(&args) {
                Ok(command) => command.execute(&paths),
                Err(e) => eprintln!("{e}"),
            },
            Err(_) => match executable_find(cmd, &paths) {
                Some(cmd) => executable_invoke(cmd, &args),
                _ => eprintln!("{cmd}: command not found"),
            },
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
            .collect::<Vec<_>>();
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

    fn execute(self, paths: &[PathBuf]) {
        match self {
            Self::Exit { code } => process::exit(code),
            Self::Echo { message } => println!("{message}"),
            Self::Type { tokens } => {
                for t in tokens {
                    match t.parse::<Command>() {
                        Ok(t) => println!("{} is a shell builtin", t.as_ref()),
                        Err(_) => match executable_find(&t, paths) {
                            Some(cmd) => println!("{t} is {}", cmd.display()),
                            _ => eprintln!("{t} not found"),
                        },
                    }
                }
            }
        }
    }
}

fn executable_invoke(cmd: PathBuf, args: &str) {
    let args = shell_words::split(args).expect("failed to parse arguments");

    let output = process::Command::new(cmd)
        .args(args)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .output()
        .unwrap();

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn executable_find(filename: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let path = Path::new(filename);
    if path.is_file() && is_executable(path) {
        return Some(path.to_path_buf());
    }

    dirs.iter().find_map(|dir| {
        dir.read_dir()
            .ok()?
            .filter_map(Result::ok)
            .find_map(|entry| {
                let path = entry.path();
                if path.file_name()? == filename && path.is_file() && is_executable(&path) {
                    return Some(path);
                }
                None
            })
    })
}

fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    #[cfg(unix)]
    {
        if let Ok(metadata) = fs::metadata(&path) {
            return metadata.permissions().mode() & 0o111 != 0;
        }
    }

    #[cfg(windows)]
    {
        if let Ok(metadata) = fs::metadata(&path) {
            return metadata.is_file(); // On Windows, executability check is simplified
        }
    }
    false
}

fn input_read(stdin: &io::Stdin) -> String {
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    input.trim().to_owned()
}
