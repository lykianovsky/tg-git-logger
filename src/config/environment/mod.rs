use once_cell::sync::Lazy;
use std::env;

pub struct Environment {
    filename: &'static str,
}

impl Environment {
    pub fn new(filename: &'static str) -> Self {
        dotenv::from_filename(filename).ok();

        return Self { filename };
    }

    pub fn get(&self, name: &str) -> String {
        let exception_message = &format!(
            "Environment: {name} is not defined in environment file {}",
            self.filename
        );

        return env::var(name).expect(exception_message);
    }

    pub fn get_or(&self, name: &str, default: &str) -> String {
        env::var(name).unwrap_or_else(|_| default.to_string())
    }
}

pub static ENV: Lazy<Environment> = Lazy::new(|| {
    return Environment::new(".env.local");
});
