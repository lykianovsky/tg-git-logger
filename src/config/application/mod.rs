use crate::config::environment::ENV;

pub struct ApplicationRedisConfig {
    pub secret: String,
    pub url: String,
}

pub struct ApplicationRabbitMqConfig {
    pub port: u16,
    pub username: String,
    pub password: String,
    pub url: String,
}

pub struct ApplicationMysqlConfig {
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub url: String,
}

pub struct ApplicationTelegramConfig {
    pub url_base: String,
    pub bot_token: String,
    pub chat_id: i64,
    pub admin_user_id: i64,
}

pub struct ApplicationConfig {
    pub port: u16,
    pub debug: bool,
    pub telegram: ApplicationTelegramConfig,
    pub mysql: ApplicationMysqlConfig,
    pub rabbit_mq: ApplicationRabbitMqConfig,
    pub redis: ApplicationRedisConfig,
}

impl ApplicationConfig {
    pub fn new() -> Self {
        let port: u16 = ENV.get("APPLICATION_PORT").parse().unwrap();
        let debug: bool = ENV.get("DEBUG").parse().unwrap();
        let telegram = Self::build_telegram_config();
        let mysql = Self::build_mysql_config();
        let rabbit_mq = Self::build_rabbit_mq_config();
        let redis = Self::build_redis_config();

        return Self {
            port,
            debug,
            telegram,
            mysql,
            rabbit_mq,
            redis,
        };
    }

    pub fn build_telegram_config() -> ApplicationTelegramConfig {
        let url_base = ENV.get("TELEGRAM_URL_BASE");
        let bot_token = ENV.get("TELEGRAM_BOT_TOKEN");
        let chat_id: i64 = ENV.get("TELEGRAM_CHAT_ID").parse().unwrap();
        let admin_user_id: i64 = ENV.get("TELEGRAM_ADMIN_USER_ID").parse().unwrap();

        return ApplicationTelegramConfig {
            url_base,
            bot_token,
            chat_id,
            admin_user_id,
        };
    }

    pub fn build_mysql_config() -> ApplicationMysqlConfig {
        let port: u16 = ENV.get("MYSQL_PORT").parse().unwrap();
        let username = ENV.get("MYSQL_USERNAME");
        let password = ENV.get("MYSQL_PASSWORD");
        let database = ENV.get("MYSQL_DATABASE_NAME");
        let url = ENV.get("MYSQL_URL");

        return ApplicationMysqlConfig {
            port,
            username,
            password,
            database,
            url,
        };
    }

    pub fn build_rabbit_mq_config() -> ApplicationRabbitMqConfig {
        let port: u16 = ENV.get("RABBITMQ_PORT").parse().unwrap();
        let username = ENV.get("RABBITMQ_USER");
        let password = ENV.get("RABBITMQ_PASSWORD");
        let url = ENV.get("RABBITMQ_URL");

        return ApplicationRabbitMqConfig {
            port,
            username,
            password,
            url,
        };
    }

    pub fn build_redis_config() -> ApplicationRedisConfig {
        let url = ENV.get("REDIS_URL");
        let secret = ENV.get("REDIS_SECRET_KEY");

        return ApplicationRedisConfig { url, secret };
    }
}
