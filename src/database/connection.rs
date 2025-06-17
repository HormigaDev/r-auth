use std::process::exit;

use colored::Colorize;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use once_cell::sync::OnceCell;
use tokio_postgres::NoTls;
use tokio_postgres::config::{Config, SslMode};
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::config::{Environment, get_config};

pub type PgPool = Pool;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),

    #[error("Deadpool Postgres error: {0}")]
    DeadpoolPool(#[from] deadpool_postgres::PoolError),

    #[error("Deadpool Postgres build error: {0}")]
    DeadpoolBuild(#[from] deadpool_postgres::BuildError),

    #[error("Configuration error: {0}")]
    Config(#[from] deadpool_postgres::ConfigError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub static GLOBAL_DB_POOL: OnceCell<PgPool> = OnceCell::new();

pub async fn initialize_global_db_pool(database_url: &str) -> Result<(), DbError> {
    let pool = create_pool(database_url).await?;
    if let Err(e) = GLOBAL_DB_POOL.set(pool) {
        eprintln!(
            "{}",
            format!(
                "{}: {:?}",
                "Ocurrió un error al conectar con la base de datos".red(),
                e
            )
        );
        exit(1);
    };
    Ok(())
}

pub async fn create_pool(database_url: &str) -> Result<PgPool, DbError> {
    let config = get_config();
    let env = &config.environment;
    let mut pg_config: Config = database_url.parse()?;

    match env {
        Environment::Development => {
            pg_config.ssl_mode(SslMode::Disable);

            let manager = Manager::from_config(
                pg_config,
                NoTls,
                ManagerConfig {
                    recycling_method: RecyclingMethod::Fast,
                },
            );

            let pool = Pool::builder(manager).max_size(10).build()?;
            Ok(pool)
        }

        Environment::Production => {
            use rustls::RootCertStore;
            use rustls::client::ClientConfig;
            use std::fs;

            // Cargar el CA autofirmado (ca.crt)
            let ca_cert = fs::read("./security/ca.crt")?;
            let mut ca_cursor = &*ca_cert;
            let ca_certs = rustls_pemfile::certs(&mut ca_cursor)
                .collect::<Result<Vec<_>, std::io::Error>>()?;

            let mut root_store = RootCertStore::empty();
            for cert in ca_certs {
                root_store.add(cert).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid CA cert")
                })?;
            }

            let tls_config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            let tls = MakeRustlsConnect::new(tls_config);

            let manager = Manager::from_config(
                pg_config,
                tls,
                ManagerConfig {
                    recycling_method: RecyclingMethod::Fast,
                },
            );

            let pool = Pool::builder(manager).max_size(10).build()?;
            Ok(pool)
        }
    }
}
