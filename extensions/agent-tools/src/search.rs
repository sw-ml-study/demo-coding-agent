//! ripgrep-backed search under one root: `.gitignore` aware, symlinks not
//! followed, output bounded by match count and bytes.

use std::collections::BTreeMap;
use std::path::Path;

use grep_regex::RegexMatcher;
use grep_searcher::sinks::UTF8;
use grep_searcher::SearcherBuilder;
use ignore::WalkBuilder;
use mlpl_extension_sdk::{OwnedError, Value};

const MAX_MATCHES: usize = 200;
const MAX_BYTES: usize = 64 * 1024;

fn string_argument<'a>(arguments: &'a [Value], index: usize, name: &str) -> Result<&'a str, OwnedError> {
    match arguments.get(index) {
        Some(Value::String(text)) => Ok(text),
        _ => Err(OwnedError::invalid_argument(format!("{name} must be a string"))),
    }
}

struct Collector {
    lines: Vec<String>,
    bytes: usize,
    truncated: bool,
}

impl Collector {
    fn push(&mut self, line: String) -> bool {
        self.bytes += line.len();
        if self.lines.len() >= MAX_MATCHES || self.bytes > MAX_BYTES {
            self.truncated = true;
            return false;
        }
        self.lines.push(line);
        true
    }

    fn search_file(&mut self, matcher: &RegexMatcher, root: &Path, path: &Path) {
        let relative = path.strip_prefix(root).unwrap_or(path).display().to_string();
        let mut searcher = SearcherBuilder::new().line_number(true).build();
        let sink = UTF8(|line_number, text| {
            Ok(self.push(format!("{relative}:{line_number}:{}", text.trim_end())))
        });
        let _ = searcher.search_path(matcher, path, sink);
    }
}

/// Collect `path:line:text` lines for `pattern` under `root`, honouring
/// `.gitignore` even outside a git repository and never following symlinks.
///
/// # Errors
/// Returns an error when the root is not a directory or the pattern is not a valid regex.
pub fn search(root: &Path, pattern: &str) -> Result<(Vec<String>, bool), OwnedError> {
    if !root.is_dir() {
        return Err(OwnedError::invalid_argument("root must be an existing directory"));
    }
    let matcher = RegexMatcher::new(pattern)
        .map_err(|error| OwnedError::invalid_argument(format!("invalid pattern: {error}")))?;
    let mut collector = Collector { lines: Vec::new(), bytes: 0, truncated: false };
    let walk = WalkBuilder::new(root).follow_links(false).require_git(false).build();
    for entry in walk.flatten() {
        if collector.truncated {
            break;
        }
        if entry.file_type().is_some_and(|kind| kind.is_file()) {
            collector.search_file(&matcher, root, entry.path());
        }
    }
    Ok((collector.lines, collector.truncated))
}

/// SDK entry: `search(root, pattern)` -> `{matches, count, truncated}`.
///
/// # Errors
/// Propagates argument and search errors as extension errors.
pub fn search_value(arguments: &[Value]) -> Result<Value, OwnedError> {
    let root = string_argument(arguments, 0, "root")?;
    let pattern = string_argument(arguments, 1, "pattern")?;
    let (lines, truncated) = search(Path::new(root), pattern)?;
    let mut record = BTreeMap::new();
    record.insert("count".to_owned(), Value::I64(i64::try_from(lines.len()).unwrap_or(i64::MAX)));
    record.insert("truncated".to_owned(), Value::Bool(truncated));
    record.insert("matches".to_owned(), Value::String(lines.join("\n")));
    Ok(Value::Record(record))
}
