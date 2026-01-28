use crate::config::environment::ENV;
use regex::Regex;

pub fn linkify(text: &str) -> String {
    let pattern = ENV.get("TASK_TRACKER_REGEXP");
    let regex = Regex::new(pattern.as_str()).unwrap();

    println!("{text}");
    regex
        .replace_all(text, |caps: &regex::Captures| {
            let original_text = &caps[0];
            let id = &caps[1];
            let link = ENV.get("TASK_TRACKER_LINK").replace("{id}", id);
            println!(
                "{original_text} {id} {link} {}",
                format!("<a href=\"{}\">{}</a>", link, original_text)
            );
            format!("<a href=\"{}\">{}</a>", link, original_text)
        })
        .to_string()
}
