use once_cell::sync::Lazy;
use std::env;

pub struct Environment {
    filename: &'static str,
}

impl Environment {
    pub fn new(filename: &'static str) -> Self {
        dotenv::from_filename(filename).ok();

        Self { filename }
    }

    pub fn get(&self, name: &str) -> String {
        let error_message = &format!(
            "Environment: {name} is not defined in environment file {}",
            self.filename
        );

        env::var(name).expect(error_message)
    }

    pub fn get_or(&self, name: &str, default: &str) -> String {
        env::var(name).unwrap_or_else(|_| default.to_string())
    }
}

pub static ENV: Lazy<Environment> = Lazy::new(|| Environment::new(".env.local"));
