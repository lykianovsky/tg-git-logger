use crate::client::task_tracker::TaskTrackerClient;
use crate::config::environment::ENV;
use regex::Regex;
use std::sync::Arc;

#[derive(Clone)]
pub struct TaskTrackerService {
    pub client: Arc<TaskTrackerClient>,
}

impl TaskTrackerService {
    pub fn new(client: Arc<TaskTrackerClient>) -> Self {
        Self { client }
    }

    pub fn extract_id(text: &str) -> Option<String> {
        let pattern = ENV.get("TASK_TRACKER_REGEXP");
        let regex = Regex::new(pattern.as_str()).ok()?;

        regex
            .captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn linkify(text: &str) -> String {
        let pattern = ENV.get("TASK_TRACKER_REGEXP");
        let regex = Regex::new(pattern.as_str()).unwrap();

        regex
            .replace_all(text, |caps: &regex::Captures| {
                let original_text = &caps[0];
                let id = &caps[1];
                let link = ENV.get("TASK_TRACKER_PATH_TO_CARD").replace("{id}", id);
                format!("<a href=\"{}\">{}</a>", link, original_text)
            })
            .to_string()
    }

    pub fn link_by_id(id: &str, text: &str) -> String {
        let link = ENV.get("TASK_TRACKER_PATH_TO_CARD").replace("{id}", id);
        format!("<a href=\"{}\">{}</a>", link, text)
    }
}
