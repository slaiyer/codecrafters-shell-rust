#[allow(unused_imports)]
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::{env, io::Stdout};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use std::{io::Stderr, process};
use std::{
    io::{self, Write},
    path::PathBuf,
};
use strum::{AsRefStr, EnumString};
use thiserror::Error;

fn main() -> rustyline::Result<()> {
    const PATH: &str = "PATH";
    let paths = match env::var(PATH) {
        Ok(ref paths) => env::split_paths(paths).collect(),
        Err(_) => {
            eprintln!("failed to parse environment variable: {PATH}");
            vec![]
        }
    };

    let mut rl = DefaultEditor::new()?;
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        stderr.flush().unwrap();
        let readline = rl.readline("$ ");
        stdout.flush().unwrap();

        match readline {
            Ok(line) => {
                let cmd = match line.split_ascii_whitespace().next() {
                    Some(cmd) => cmd,
                    _ => continue,
                };
                handle_input(&line, cmd, &paths, &mut stdout, &mut stderr);
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("^D");
                break;
            }
            Err(err) => {
                eprintln!("error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn handle_input(
    line: &str,
    cmd: &str,
    paths: &[PathBuf],
    stdout: &mut Stdout,
    stderr: &mut Stderr,
) {
    let args = line
        .chars()
        .skip(cmd.len())
        .collect::<String>()
        .trim()
        .to_owned();

    match cmd.parse::<Command>() {
        Ok(command) => match command.build(&args) {
            Ok(command) => command.execute(paths),
            Err(e) => eprintln!("{e}"),
        },
        Err(_) => match executable_find(cmd, paths) {
            Some(cmd) => executable_invoke(cmd, &args, stdout, stderr),
            _ => eprintln!("{cmd}: command not found"),
        },
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
            Self::Exit { .. } => build_command_exit(tokens),
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
            Self::Type { tokens } => get_command_types(tokens, paths),
        }
    }
}

fn build_command_exit(tokens: Vec<String>) -> Result<Command, CommandError> {
    match tokens.len() {
        n if n > 1 => Err(CommandError::Argument("too many supplied".to_owned())),
        1 => Ok(Command::Exit {
            code: match tokens[0].parse::<i32>() {
                Ok(code) => code,
                _ => 1,
            },
        }),
        _ => Ok(Command::Exit { code: 0 }),
    }
}

fn get_command_types(tokens: Vec<String>, paths: &[PathBuf]) {
    tokens.into_iter().for_each(|t| match t.parse::<Command>() {
        Ok(t) => println!("{} is a shell builtin", t.as_ref()),
        Err(_) => match executable_find(&t, paths) {
            Some(cmd) => println!(
                "{t} is {}",
                match cmd.canonicalize() {
                    Ok(path) => path.display().to_string(),
                    _ => cmd.to_string_lossy().into_owned(),
                }
            ),
            _ => eprintln!("{t} not found"),
        },
    })
}

fn executable_invoke(cmd: PathBuf, args: &str, stdout: &mut Stdout, stderr: &mut Stderr) {
    let args = match shell_words::split(args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("failed to parse arguments: {e}");
            return;
        }
    };

    let output = process::Command::new(cmd)
        .args(args)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .output()
        .unwrap();

    stdout.write_all(&output.stdout).unwrap();
    stderr.write_all(&output.stderr).unwrap();
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
            return metadata.is_file(); // TODO: consider https://docs.rs/is_executable/latest/src/is_executable/lib.rs.html#146
        }
    }
    false
}
