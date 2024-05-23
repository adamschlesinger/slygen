//! Ask a question and get an answer.

use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::process::exit;

use crossterm::{event, execute, QueueableCommand};
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::event::{Event, KeyCode};
use crossterm::style::{Color, Print, SetForegroundColor};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::{debug, error};
use crate::terminal::timestamp;

/// Ask a question and get an answer.
/// # Open ended:
/// ```rust
/// let choice = question!("What is your name?");
/// ```
///
/// # With a default:
/// ```rust
/// let choice = question!(
///     "What is the answer to life, the universe, and everything?",
///     "42"
/// );
/// ```
///
/// # With defined options:
/// ``` rust
/// question!("Would you like to continue?" {
///     "yes" => todo!(),
///     "no" => todo!()
/// });
/// ```
///
/// # With single character options:
/// ``` rust
/// question!("Would you like to continue?" {
///     'y' => todo!(),
///     'n' => todo!()
/// });
/// ```
#[macro_export]
macro_rules! question {
    ($msg:tt {
        $($opt:tt => $response:expr),+$(,)*
    }) => {
        let answer = $crate::terminal::question::Question::new($msg)
            $(.with_option($opt))+
            .ask();

        match answer.as_str() {
            $($opt => $response),*
            &_ => {
                $crate::error!("Invalid Input");
                std::process::exit(exitcode::USAGE);
            }
        }
    };
    ($msg:tt, $default:expr) => {{
        $crate::terminal::question::Question::new($msg)
            .with_default($default)
            .ask()
    }};
    ($msg:tt) => {{
        $crate::terminal::question::Question::new(format!($msg))
            .ask()
    }};
}

/// Ask a question and get an answer.
pub struct Question {
    text: String,
    default: Option<String>,
    options: Vec<String>,
}

impl Question {
    /// todo
    pub fn new(text: &str) -> Self {
        Self {
            text: text.into(),
            default: None,
            options: vec![],
        }
    }
    /// todo
    pub fn with_default(&mut self, default: &str) -> &mut Self {
        self.default = Some(default.into());
        self
    }

    /// todo
    pub fn with_option(&mut self, option: &str) -> &mut Self {
        self.options.push(option.into());
        self
    }

    /// todo
    pub fn ask(&self) -> String {
        print_question(self)
            .expect("Could not print question.");

        get_input(self)
            
        // if self.options.iter().all(|opt| opt.len() == 1) {
        //     let opt_chars: Vec<char> = self.options.iter()
        //         .map(|opt| opt.chars()
        //             .next()
        //             .unwrap())
        //         .collect();
        // 
        //     get_input_char(&opt_chars)
        //         .expect("Failed to get input")
        // } else {
        //     get_input(self)
        // }
    }
}

fn print_question(question: &Question) -> Result<(), std::io::Error> {
    let mut stdout = stdout();
    stdout.queue(SetForegroundColor(Color::Grey))?;
    stdout.queue(Print(timestamp()))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(Print(format!(" {} ", &question.text)))?;

    if !question.options.is_empty() {
        stdout.queue(Print("("))?;
        for option in &question.options {
            stdout.queue(Print(option))?;

            if question.options.last().unwrap() != option {
                stdout.queue(Print("/"))?;
            }
        }
        stdout.queue(Print(")"))?;
    } else if let Some(default) = &question.default {
        stdout.queue(Print(format!("Hit ENTER to use default ({default})")))?;
    }

    stdout.queue(Print("\n"))?;
    stdout.queue(SetForegroundColor(Color::Grey))?;
    stdout.queue(Print(format!("{} ", timestamp())))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.flush()?;

    Ok(())
}

fn get_input(question: &Question) -> String {
    let mut input = String::new();
    stdin().read_line(&mut input)
        .unwrap_or_else(|err| {
            error!("{:?}", err);
            exit(exitcode::USAGE);
        });

    if let Some(default) = &question.default {
        if input == "\n" {
            input = default.clone();
            let _ = execute!(
                stdout(),
                MoveUp(1),
                MoveToColumn(timestamp().len() as u16),
                Print(format!(" {input}\n")),
            );
        }
    }

    debug!("Input is {:?}", input);
    input.trim().to_string()
}

fn get_input_char(options: &[char]) -> Result<String, Box<dyn Error>> {
    enable_raw_mode()?;

    let event = loop {
        if let Event::Key(event) = event::read()? {
            break event;
        }
    };

    disable_raw_mode()?;

    for opt in options {
        if event.code == KeyCode::Char(*opt) {
            return Ok(opt.to_string());
        }
    }

    let _ = execute!(
        stdout(),
        Print(format!("{:?}\n", event.code)),
    );

    error!("Invalid Input");
    exit(exitcode::USAGE);
}
