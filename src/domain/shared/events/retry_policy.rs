pub struct RetryPolicy {
    pub max_attempts: i64,
    pub delay_ms: u32,
}

impl RetryPolicy {
    pub fn new(max_attempts: i64, delay_ms: u32) -> Self {
        Self {
            max_attempts,
            delay_ms,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay_ms: 5000,
        }
    }
}
