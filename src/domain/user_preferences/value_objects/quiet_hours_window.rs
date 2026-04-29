use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuietHoursWindow {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl QuietHoursWindow {
    pub fn new(start: NaiveTime, end: NaiveTime) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, time: NaiveTime) -> bool {
        if self.start == self.end {
            return false;
        }
        if self.start < self.end {
            time >= self.start && time < self.end
        } else {
            time >= self.start || time < self.end
        }
    }
}
