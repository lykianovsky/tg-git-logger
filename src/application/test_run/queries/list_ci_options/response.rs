use crate::domain::test_run::ports::test_runner::CiOption;

pub struct ListCiOptionsResponse {
    pub options: Vec<CiOption>,
}
