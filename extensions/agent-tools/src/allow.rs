//! The fixed command allow-list. Matching is on argv words, never on a
//! shell string, so the model can never reach `sh -c`.

const ALLOWED: [[&str; 2]; 6] = [
    ["cargo", "test"],
    ["cargo", "check"],
    ["cargo", "clippy"],
    ["cargo", "fmt"],
    ["git", "diff"],
    ["git", "status"],
];

/// Accept argv only when its first two words are an allow-listed pair.
///
/// # Errors
/// Returns the refused command text when the prefix is not allow-listed.
pub fn allowed(argv: &[String]) -> Result<(), String> {
    let prefix_ok = argv.len() >= 2
        && ALLOWED
            .iter()
            .any(|pair| pair[0] == argv[0] && pair[1] == argv[1]);
    if prefix_ok {
        Ok(())
    } else {
        Err(format!("run: command not allowed: {}", argv.join(" ")))
    }
}

#[cfg(test)]
mod tests {
    use super::allowed;

    fn words(command: &str) -> Vec<String> {
        command.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn allow_listed_prefixes_pass_with_any_arguments() {
        for command in ["cargo test", "cargo test --lib -- --nocapture", "git diff HEAD~1", "git status --short"] {
            assert!(allowed(&words(command)).is_ok(), "{command}");
        }
    }

    #[test]
    fn everything_else_is_refused_by_name() {
        for command in ["", "cargo", "cargo run", "git push", "sh -c cargo test", "rm -rf .", "mlpl x.mlpl"] {
            let refused = allowed(&words(command)).unwrap_err();
            assert!(refused.starts_with("run: command not allowed: "), "{command}: {refused}");
        }
    }
}
