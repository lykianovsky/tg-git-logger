use crate::domain::repository::value_objects::repository_id::RepositoryId;
use sha2::{Digest, Sha256};

/// Отпечаток упавшего теста: по нему ищется уже заведённая карточка в трекере.
/// Берётся от репозитория, проекта, файла и названия — они стабильны между прогонами,
/// в отличие от текста ошибки, который плавает (таймауты, разные значения в сообщении).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestFingerprint(pub String);

impl TestFingerprint {
    pub fn build(repository_id: RepositoryId, project: &str, file: &str, title: &str) -> Self {
        let mut hasher = Sha256::new();

        hasher.update(repository_id.0.to_string().as_bytes());
        hasher.update([0]);
        hasher.update(project.as_bytes());
        hasher.update([0]);
        hasher.update(file.as_bytes());
        hasher.update([0]);
        hasher.update(title.as_bytes());

        Self(hex::encode(hasher.finalize()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_test_gives_same_fingerprint() {
        let first = TestFingerprint::build(RepositoryId(1), "accounts", "tests/a.spec.ts", "вход");
        let second = TestFingerprint::build(RepositoryId(1), "accounts", "tests/a.spec.ts", "вход");

        assert_eq!(first, second);
    }

    #[test]
    fn same_title_in_other_project_gives_other_fingerprint() {
        let accounts =
            TestFingerprint::build(RepositoryId(1), "accounts", "tests/a.spec.ts", "вход");
        let landing = TestFingerprint::build(RepositoryId(1), "landing", "tests/a.spec.ts", "вход");

        assert_ne!(accounts, landing);
    }

    #[test]
    fn same_test_in_other_repository_gives_other_fingerprint() {
        let first = TestFingerprint::build(RepositoryId(1), "accounts", "tests/a.spec.ts", "вход");
        let second = TestFingerprint::build(RepositoryId(2), "accounts", "tests/a.spec.ts", "вход");

        assert_ne!(first, second);
    }
}
