use crate::config::environment::ENV;

pub struct ApplicationSecretConfig {
    pub reversible_cipher_secret: String,
}

pub struct ApplicationGithubConfig {
    pub base: String,
    pub api_base: String,
    pub repository: String,
    pub repository_owner: String,
    pub oauth_pathname: String,
    pub oauth_client_scope: String,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
}

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
    pub github: ApplicationGithubConfig,
    pub secret: ApplicationSecretConfig,
}

impl ApplicationConfig {
    pub fn new() -> Self {
        let port: u16 = ENV.get("APPLICATION_PORT").parse().unwrap();
        let debug: bool = ENV.get("DEBUG").parse().unwrap();
        let telegram = Self::build_telegram_config();
        let mysql = Self::build_mysql_config();
        let rabbit_mq = Self::build_rabbit_mq_config();
        let redis = Self::build_redis_config();
        let github = Self::build_github_config();
        let secret = Self::build_secret_config();

        Self {
            port,
            debug,
            telegram,
            mysql,
            rabbit_mq,
            redis,
            github,
            secret,
        }
    }

    pub fn build_telegram_config() -> ApplicationTelegramConfig {
        let url_base = ENV.get("TELEGRAM_URL_BASE");
        let bot_token = ENV.get("TELEGRAM_BOT_TOKEN");
        let chat_id: i64 = ENV.get("TELEGRAM_CHAT_ID").parse().unwrap();
        let admin_user_id: i64 = ENV.get("TELEGRAM_ADMIN_USER_ID").parse().unwrap();

        ApplicationTelegramConfig {
            url_base,
            bot_token,
            chat_id,
            admin_user_id,
        }
    }

    pub fn build_mysql_config() -> ApplicationMysqlConfig {
        let port: u16 = ENV.get("MYSQL_PORT").parse().unwrap();
        let username = ENV.get("MYSQL_USERNAME");
        let password = ENV.get("MYSQL_PASSWORD");
        let database = ENV.get("MYSQL_DATABASE_NAME");
        let url = ENV.get("MYSQL_URL");

        ApplicationMysqlConfig {
            port,
            username,
            password,
            database,
            url,
        }
    }

    pub fn build_rabbit_mq_config() -> ApplicationRabbitMqConfig {
        let port: u16 = ENV.get("RABBITMQ_PORT").parse().unwrap();
        let username = ENV.get("RABBITMQ_USER");
        let password = ENV.get("RABBITMQ_PASSWORD");
        let url = ENV.get("RABBITMQ_URL");

        ApplicationRabbitMqConfig {
            port,
            username,
            password,
            url,
        }
    }

    pub fn build_redis_config() -> ApplicationRedisConfig {
        let url = ENV.get("REDIS_URL");
        let secret = ENV.get("REDIS_SECRET_KEY");

        ApplicationRedisConfig { url, secret }
    }

    pub fn build_github_config() -> ApplicationGithubConfig {
        let base = ENV.get("GITHUB_BASE");
        let api_base = ENV.get("GITHUB_API_BASE");

        let repository = ENV.get("GITHUB_REPOSITORY");
        let repository_owner = ENV.get("GITHUB_REPOSITORY_OWNER");

        let oauth_pathname = ENV.get("GITHUB_OAUTH_PATHNAME");
        let oauth_client_scope = ENV.get("GITHUB_OAUTH_CLIENT_SCOPE");
        let oauth_client_id = ENV.get("GITHUB_OAUTH_CLIENT_ID");
        let oauth_client_secret = ENV.get("GITHUB_OAUTH_CLIENT_SECRET");

        ApplicationGithubConfig {
            base,
            api_base,
            repository,
            repository_owner,
            oauth_pathname,
            oauth_client_scope,
            oauth_client_id,
            oauth_client_secret,
        }
    }

    pub fn build_secret_config() -> ApplicationSecretConfig {
        let reversible_cipher_secret = ENV.get("REVERSABLE_CIPHER_SECRET_KEY");

        ApplicationSecretConfig {
            reversible_cipher_secret,
        }
    }
}
