use crate::domain::task::value_objects::task_id::TaskId;

pub trait TaskTrackerService: Send + Sync {
    fn extract_task_id_by_pattern(&self, text: &str) -> Option<TaskId>;

    /// Возвращает (совпавший фрагмент текста, task_id), если паттерн сработал.
    fn extract_task_match(&self, text: &str) -> Option<(String, TaskId)>;

    /// Возвращает первое совпадение (matched_fragment, task_id) используя явно указанный паттерн.
    fn extract_match_with_pattern(&self, text: &str, pattern: &str) -> Option<(String, TaskId)>;

    /// Возвращает все совпадения (matched_fragment, task_id) используя явно указанный паттерн.
    fn extract_all_matches_with_pattern(&self, text: &str, pattern: &str) -> Vec<(String, TaskId)>;
}
