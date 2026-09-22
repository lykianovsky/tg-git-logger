/// Метка запуска: бот передаёт её во вход workflow, CI подставляет в имя прогона,
/// и по ней прогон находится обратно — `workflow_dispatch` не возвращает идентификатор.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunTag(pub String);

/// Длина случайной части метки: её достаточно, чтобы метки не совпали, и она
/// помещается в имя прогона рядом с веткой
const RANDOM_PART_BYTES: usize = 8;

impl RunTag {
    /// Метка вида `20260922-1a2b3c4d5e6f7a8b`: по дате её легко узнать в списке прогонов,
    /// случайная часть исключает совпадение при двух запусках в одну секунду
    pub fn generate(now: chrono::DateTime<chrono::Utc>) -> Self {
        use rand::RngCore;

        let mut random = [0u8; RANDOM_PART_BYTES];

        rand::rngs::OsRng.fill_bytes(&mut random);

        Self(format!("{}-{}", now.format("%Y%m%d"), hex::encode(random)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RunTag {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tags_do_not_repeat() {
        let now = chrono::Utc::now();
        let first = RunTag::generate(now);
        let second = RunTag::generate(now);

        assert_ne!(first, second);
    }

    #[test]
    fn tag_starts_with_date() {
        let now = chrono::Utc::now();
        let tag = RunTag::generate(now);

        assert!(tag.as_str().starts_with(&now.format("%Y%m%d").to_string()));
    }
}
