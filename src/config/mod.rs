use colored::Colorize;
use once_cell::sync::OnceCell;
use std::process::exit;

const PASSWORD_HASH_MEMORY_COST: &str = "PASSWORD_HASH_MEMORY_COST";
const PASSWORD_HASH_TIME_COST: &str = "PASSWORD_HASH_TIME_COST";
const PASSWORD_HASH_LANES: &str = "PASSWORD_HASH_LANES";
const PASSWORD_HASH_LENGTH: &str = "PASSWORD_HASH_LENGTH";
const JWT_SECRET: &str = "JWT_SECRET";
const DATABASE_URL: &str = "DATABASE_URL";
const RUST_ENVIRONMENT: &str = "RUST_ENVIRONMENT";

pub struct PasswordHashingConfig {
    pub hash_length: u32,
    pub mem_cost: u32,
    pub time_cost: u32,
    pub lanes: u32,
}

pub struct AuthConfig {
    pub secret: String,
}

pub struct DbConfig {
    pub database_url: String,
}

pub struct AppConfig {
    pub password: PasswordHashingConfig,
    pub auth: AuthConfig,
    pub db: DbConfig,
    pub environment: Environment,
}

#[derive(Debug)]
pub enum Environment {
    Development,
    Production,
}

static CONFIG: OnceCell<AppConfig> = OnceCell::new();

pub fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    let mem_cost = get_env_number(PASSWORD_HASH_MEMORY_COST);
    let time_cost = get_env_number(PASSWORD_HASH_TIME_COST);
    let lanes = get_env_number(PASSWORD_HASH_LANES);
    let hash_length = get_env_number(PASSWORD_HASH_LENGTH);
    let jwt_secret = get_env(JWT_SECRET);
    let database_url = get_env(DATABASE_URL);
    let environment = match get_env(RUST_ENVIRONMENT).as_str() {
        "production" => Environment::Production,
        _ => Environment::Development,
    };

    let config = AppConfig {
        password: PasswordHashingConfig {
            hash_length,
            mem_cost,
            time_cost,
            lanes,
        },
        auth: AuthConfig { secret: jwt_secret },
        db: DbConfig { database_url },
        environment,
    };

    CONFIG.set(config).map_err(|_| "Config ya inicializada")?;
    Ok(())
}

pub fn get_config() -> &'static AppConfig {
    CONFIG.get().unwrap_or_else(|| {
        eprintln!("{}", "Configurations is not initialized");
        exit(1);
    })
}

fn get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        eprintln!(
            "{}",
            format!("The env variable {} is not configured", key.yellow()).red()
        );
        exit(1);
    })
}

fn get_env_number(key: &str) -> u32 {
    let variable: u32 = get_env(key).parse().unwrap_or_else(|_| {
        eprintln!(
            "{}",
            format!("Error parsing the env var {} to number", key.yellow()).red()
        );
        exit(1);
    });

    variable
}
