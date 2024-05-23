use std::error::Error;
use std::fmt::{Display, Formatter};
use std::process::Command;

/// Executes the cmd and returns when done.
#[macro_export]
macro_rules! sh {
    ($arg:expr) => {
        $crate::shell::__sh(format!($arg))
    };
}

/// Executes the cmd and returns when done. Exits on error.
#[macro_export]
macro_rules! sh_exit {
    ($arg:expr) => {
        $crate::shell::__sh(format!($arg))
            .unwrap_or_else(|err| std::process::exit(err.code))
    };
}

/// Describes when a shell command has failed with a non-zero exit code
#[derive(Debug)]
pub struct ShellError {
    /// Exit code
    pub code: i32,

    /// Output from the command
    pub output: String,
}

impl Error for ShellError {}
impl Display for ShellError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.output)
    }
}


/// todo
pub type Result = core::result::Result<String, ShellError>;

#[doc(hidden)]
pub fn __sh(cmd: String) -> Result {
    let cmd_result = Command::new("sh").arg("-c").arg(cmd).output();

    let cmd_output = cmd_result.unwrap();
    let std_bytes = if cmd_output.status.success() {
        cmd_output.stdout
    } else {
        cmd_output.stderr
    };

    let code = cmd_output.status.code().unwrap_or(0);
    let output = String::from_utf8(std_bytes).unwrap().trim().to_string();

    if !cmd_output.status.success() {
        return Err(ShellError { code, output });
    }

    Ok(output)
}
