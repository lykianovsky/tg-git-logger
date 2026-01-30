use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KaitenCard {
    pub column_id: u64,
}
