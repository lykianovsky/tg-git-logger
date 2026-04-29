use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static GITHUB_MENTION_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@([A-Za-z0-9](?:[A-Za-z0-9-]{0,38}))").unwrap());

const MAX_MENTIONS: usize = 20;

/// Extract unique GitHub-style @-mentions from text.
///
/// Returns logins without the leading `@`. Limited to first MAX_MENTIONS
/// distinct logins (case-insensitive dedup, preserves first-seen casing).
pub fn extract_github_mentions(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for cap in GITHUB_MENTION_RE.captures_iter(text) {
        let login = match cap.get(1) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };
        let key = login.to_lowercase();
        if seen.insert(key) {
            result.push(login);
            if result.len() >= MAX_MENTIONS {
                break;
            }
        }
    }

    result
}
