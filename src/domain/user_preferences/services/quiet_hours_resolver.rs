use crate::domain::user_preferences::entities::user_preferences::UserPreferences;
use crate::domain::user_preferences::value_objects::quiet_hours_window::QuietHoursWindow;
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::Tz;

pub struct QuietHoursResolver {
    default_window: QuietHoursWindow,
    default_timezone: Tz,
}

impl QuietHoursResolver {
    pub fn new(default_window: QuietHoursWindow, default_timezone: Tz) -> Self {
        Self {
            default_window,
            default_timezone,
        }
    }

    pub fn is_quiet(&self, prefs: Option<&UserPreferences>, now: DateTime<Utc>) -> bool {
        if let Some(p) = prefs {
            if let Some(snooze_until) = p.snooze_until {
                if now < snooze_until {
                    return true;
                }
            }
            if let Some(vacation_until) = p.vacation_until {
                if now <= vacation_until {
                    return true;
                }
            }
        }

        let timezone = prefs
            .and_then(|p| p.timezone)
            .unwrap_or(self.default_timezone);
        let window = prefs
            .and_then(|p| p.dnd_window)
            .unwrap_or(self.default_window);

        let local_time = now.with_timezone(&timezone).time();
        window.contains(local_time)
    }

    pub fn next_active_at(
        &self,
        prefs: Option<&UserPreferences>,
        now: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let mut start = now;

        if let Some(p) = prefs {
            if let Some(snooze_until) = p.snooze_until {
                if snooze_until > start {
                    start = snooze_until;
                }
            }
            if let Some(vacation_until) = p.vacation_until {
                if vacation_until > start {
                    start = vacation_until;
                }
            }
        }

        let timezone = prefs
            .and_then(|p| p.timezone)
            .unwrap_or(self.default_timezone);
        let window = prefs
            .and_then(|p| p.dnd_window)
            .unwrap_or(self.default_window);

        let local_start = start.with_timezone(&timezone);

        if !window.contains(local_start.time()) {
            return start;
        }

        let local_date = local_start.date_naive();
        let end_today_naive = local_date.and_time(window.end);

        let end_dt = match timezone.from_local_datetime(&end_today_naive).single() {
            Some(dt) if dt > local_start => dt,
            _ => {
                let next_date = local_date.succ_opt().unwrap_or(local_date);
                let end_tomorrow = next_date.and_time(window.end);
                timezone
                    .from_local_datetime(&end_tomorrow)
                    .single()
                    .unwrap_or_else(|| local_start + Duration::hours(24))
            }
        };

        end_dt.with_timezone(&Utc)
    }
}
