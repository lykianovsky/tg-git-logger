use crate::domain::shared::date::range::DateRange;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumString, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BuildVersionControlDateRangeReportExecutorCommandForWho {
    Me,
    Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildVersionControlDateRangeReportExecutorCommand {
    pub social_user_id: SocialUserId,
    pub date_range: DateRange,
    pub for_who: BuildVersionControlDateRangeReportExecutorCommandForWho,
}
