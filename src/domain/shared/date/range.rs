use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DateRange {
    pub since: DateTime<Utc>,
    pub until: DateTime<Utc>,
}

impl DateRange {
    pub fn new(since: DateTime<Utc>, until: DateTime<Utc>) -> Self {
        Self { since, until }
    }

    pub fn last_week() -> Self {
        let until = Utc::now();
        Self {
            since: until - chrono::Duration::weeks(1),
            until,
        }
    }

    pub fn last_2_weeks() -> Self {
        let until = Utc::now();
        Self {
            since: until - chrono::Duration::weeks(2),
            until,
        }
    }

    pub fn last_n_days(days: i64) -> Self {
        let until = Utc::now();
        Self {
            since: until - chrono::Duration::days(days),
            until,
        }
    }

    pub fn last_month() -> Self {
        let until = Utc::now();
        Self {
            since: until - chrono::Duration::days(30),
            until,
        }
    }

    pub fn this_month() -> Self {
        let now = Utc::now();
        let since = Utc
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
            .single()
            .expect("start of month is always valid in UTC");

        Self { since, until: now }
    }
}
