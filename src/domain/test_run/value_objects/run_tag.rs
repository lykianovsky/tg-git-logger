/// Метка запуска: бот передаёт её во вход workflow, CI подставляет в имя прогона,
/// и по ней прогон находится обратно — `workflow_dispatch` не возвращает идентификатор.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunTag(pub String);

impl RunTag {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RunTag {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
