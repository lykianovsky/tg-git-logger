use crate::domain::shared::date::range::DateRange;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub enum BuildVersionControlDateRangeReportExecutorCommandForWho {
    Me,
    Repository,
}

pub struct BuildVersionControlDateRangeReportExecutorCommand {
    pub social_user_id: SocialUserId,
    pub date_range: DateRange,
    pub for_who: BuildVersionControlDateRangeReportExecutorCommandForWho,
}
