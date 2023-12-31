use crate::{ReplInputEvent, ReplState};

use super::{consts::PRQLITE_VERSION, traits::Runner};
use anyhow::Result;

use prql_compiler::PRQL_VERSION;

use std::io::{stdout, Stdout, Write};

use crossterm::event::{read, Event, KeyCode, KeyEvent};

lazy_static! {
    static ref WELCOME_MSG: String = {
        format!(
            r#"
                     Welcome to PRQLite!   
type ".help" to show avaliable commands, or start typing queries.
PRQL version: {:?}
Prqlite version: {}
    "#,
            PRQL_VERSION.to_string(),
            PRQLITE_VERSION
        )
    };
}

pub struct NormalRepl<'a> {
    prompt: String,
    command_prefix: String,
    state: &'a ReplState,
}

impl<'a> NormalRepl<'a> {
    pub fn new<T: ToString>(prompt: T, command_prefix: T, state: &'a ReplState) -> Self {
        Self {
            prompt: prompt.to_string(),
            command_prefix: command_prefix.to_string(),
            state,
        }
    }
}

impl<'a> Runner for NormalRepl<'a> {
    fn run(&self) -> Result<()> {
        let repl_input_event = ReplInputEvent::new(self.state);
        let mut stdout = stdout();
        let mut buf = String::new();

        println!("{}", *WELCOME_MSG);

        match self.state.get_prqlite_conn().unwrap().get_conn() {
            Some("") => println!("Connected to a transient in-memory database."),
            None => {
                panic!("Database is unknown or not exist.")
            }
            _ => {}
        }
        loop {
            read_input(&mut stdout, &mut buf, self.prompt.clone())?;

            buf = buf
                .trim()
                .to_owned()
                .replacen(";", "", buf.rfind(";").unwrap());

            let exec_output = match buf.trim().starts_with(&self.command_prefix) {
                true => repl_input_event.on_command(&buf),
                false => repl_input_event.on_regular_input(&buf),
            };

            if let Err(err) = exec_output {
                eprintln!("\x1b[93m{}\x1b[0m", err.to_string());
            } else {
                println!("{}", exec_output.unwrap());
            }

            buf.clear();
        }
    }
}

pub fn read_input(stdout: &mut Stdout, buf: &mut String, prompt: String) -> Result<()> {
    print!("{} ", prompt);
    stdout.flush().unwrap();

    while let Event::Key(KeyEvent { code, .. }) = read()? {
        match code {
            KeyCode::Enter => {
                if buf.trim().chars().last().unwrap() != ';' {
                    buf.push_str("\n");
                    print!("..~");
                    stdout.flush()?;
                    continue;
                }
                break;
            }
            KeyCode::Char(c) => {
                buf.push(c);
            }
            _ => {}
        }
    }

    Ok(())
}
