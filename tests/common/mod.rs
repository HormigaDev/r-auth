use colored::Colorize;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::env;
use tokio_postgres::NoTls;
use tokio_postgres::config::{Config, SslMode};

pub type PgPool = Pool;

pub static TEST_DB_POOL: Lazy<PgPool> = Lazy::new(|| {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL_TEST").expect("TEST ERROR: La variable de entorno de la URL de conexión a la base de pruebas de datos no está configurada.");

    let mut pg_config: Config = database_url
        .parse()
        .expect("TEST ERROR: la URL de la base de datos de pruebas no es válida.");
    pg_config.ssl_mode(SslMode::Disable);

    let manager = Manager::from_config(
        pg_config,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );

    let pool = Pool::builder(manager).max_size(5).build().expect(
        "TEST ERROR: No se pudo construir el pool de conexiones a la base de datos de pruebas.",
    );
    println!(
        "{} {}",
        "Base de datos de prueba inicializada:".green(),
        database_url
    );

    pool
});

pub fn get_test_pool() -> &'static PgPool {
    &TEST_DB_POOL
}

pub async fn setup_test_environment(pool: &PgPool) {
    let client = pool
        .get()
        .await
        .expect("TEST ERROR: Error al obtener el cliente");
    client
        .query(
            r#"
            TRUNCATE TABLE users RESTART IDENTITY CASCADE;
        "#,
            &[],
        )
        .await
        .expect("TEST ERROR: Error al limpiar la base de datos de pruebas");
}
