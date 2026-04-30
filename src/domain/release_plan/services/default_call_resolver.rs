use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;

pub struct DefaultCallResolver {
    pub default_weekday: Weekday,
    pub default_time_msk: NaiveTime,
    pub timezone: Tz,
}

impl DefaultCallResolver {
    pub fn new(default_weekday: Weekday, default_time_msk: NaiveTime, timezone: Tz) -> Self {
        Self {
            default_weekday,
            default_time_msk,
            timezone,
        }
    }

    /// Returns datetime of default release call in UTC, computed as the most recent
    /// `default_weekday` on or before `planned_date`, at `default_time_msk` in `timezone`.
    pub fn resolve_for(&self, planned_date: NaiveDate) -> DateTime<Utc> {
        let mut date = planned_date;
        for _ in 0..7 {
            if date.weekday() == self.default_weekday {
                break;
            }
            date = date.checked_sub_days(Days::new(1)).unwrap_or(date);
        }

        let local = self
            .timezone
            .from_local_datetime(&date.and_time(self.default_time_msk))
            .single()
            .unwrap_or_else(|| {
                self.timezone
                    .from_local_datetime(&date.and_time(self.default_time_msk))
                    .earliest()
                    .expect("ambiguous local datetime without earliest")
            });

        local.with_timezone(&Utc)
    }
}
