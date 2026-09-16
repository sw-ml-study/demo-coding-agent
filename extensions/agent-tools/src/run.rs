//! Allow-listed process execution inside one root with a timeout and
//! captured output. Never a shell.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use mlpl_extension_sdk::{OwnedError, Value};

use crate::allow::allowed;

const TIMEOUT: Duration = Duration::from_secs(120);
const MAX_OUTPUT: usize = 64 * 1024;

/// Outcome of one bounded run.
pub struct Outcome {
    pub status: i64,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

fn capture(reader: Option<impl Read + Send + 'static>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(reader) = reader {
            let _ = reader.take(MAX_OUTPUT as u64).read_to_string(&mut text);
        }
        text
    })
}

fn wait_bounded(child: &mut Child, deadline: Instant) -> (i64, bool) {
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return (i64::from(status.code().unwrap_or(-1)), false);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return (-1, true);
        }
        thread::sleep(Duration::from_millis(25));
    }
}

/// Run an allow-listed argv in `root`, capturing bounded output.
///
/// # Errors
/// Returns an error when the root is not a directory, the command is refused, or it cannot start.
pub fn run(root: &Path, argv: &[String]) -> Result<Outcome, OwnedError> {
    if !root.is_dir() {
        return Err(OwnedError::invalid_argument("root must be an existing directory"));
    }
    allowed(argv).map_err(OwnedError::invalid_argument)?;
    let mut child = Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| OwnedError::extension(format!("run: cannot start {}: {error}", argv[0])))?;
    let stdout = capture(child.stdout.take());
    let stderr = capture(child.stderr.take());
    let (status, timed_out) = wait_bounded(&mut child, Instant::now() + TIMEOUT);
    Ok(Outcome {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
        timed_out,
    })
}

/// SDK entry: `run(root, command)` -> `{status, stdout, stderr, timed_out}`.
///
/// # Errors
/// Propagates argument, allow-list, and spawn errors as extension errors.
pub fn run_value(arguments: &[Value]) -> Result<Value, OwnedError> {
    let (Some(Value::String(root)), Some(Value::String(command))) = (arguments.first(), arguments.get(1)) else {
        return Err(OwnedError::invalid_argument("root and command must be strings"));
    };
    let argv: Vec<String> = command.split_whitespace().map(str::to_owned).collect();
    let outcome = run(Path::new(root), &argv)?;
    let mut record = BTreeMap::new();
    record.insert("status".to_owned(), Value::I64(outcome.status));
    record.insert("stdout".to_owned(), Value::String(outcome.stdout));
    record.insert("stderr".to_owned(), Value::String(outcome.stderr));
    record.insert("timed_out".to_owned(), Value::Bool(outcome.timed_out));
    Ok(Value::Record(record))
}
