use std::time::Duration;
use std::str::FromStr;

use sqlx::{
    pool::{Pool, PoolOptions}, Connection, Database, Error as SqlxError
};
use tracing::{
    log::LevelFilter,
    event,
    span,
    Level,
};

#[cfg(feature = "mysql")]
use sqlx::mysql::{MySql, MySqlConnectOptions};

#[cfg(feature = "postgres")]
use sqlx::postgres::{Postgres, PgConnectOptions};

#[cfg(feature = "sqlite")]
use sqlx::sqlite::{Sqlite, SqliteConnectOptions};

#[derive(Debug, Clone)]
pub(crate) enum ConnectionOptions {
    #[cfg(feature = "mysql")]
    MySql(MySqlConnectOptions),

    #[cfg(feature = "postgres")]
    Postgres(PgConnectOptions),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnectOptions),
}

impl ConnectionOptions {
    pub(crate) fn from_url(url: &str) -> Result<Self, SqlxError> {
        #[cfg(feature = "mysql")]
        if url.starts_with("mysql://") {
            return Ok(ConnectionOptions::MySql(MySqlConnectOptions::from_str(
                url,
            )?));
        }

        #[cfg(feature = "postgres")]
        if url.starts_with("postgres://") {
            return Ok(ConnectionOptions::Postgres(
                PgConnectOptions::from_str(url)?,
            ));
        }

        #[cfg(feature = "sqlite")]
        if url.starts_with("sqlite://") {
            return Ok(ConnectionOptions::Sqlite(
                SqliteConnectOptions::from_str(url)?,
            ));
        }

        Err(SqlxError::Configuration(
            format!("Invalid connection {}", url).into(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    connect_options: ConnectionOptions,
    max_connections: u32,
    min_connections: u32,
    acquire_timeout: Duration,
    acquire_slow_threshold: Duration,
    acquire_slow_level: LevelFilter,
}

impl Options {
    pub fn from_url(url: &str) -> Result<Self, SqlxError> {
        let connect_options = ConnectionOptions::from_url(url)?;
        Ok(Self {
            connect_options,
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(10),
            acquire_slow_threshold: Duration::from_secs(1),
            acquire_slow_level: LevelFilter::Warn,
        })
    }

    pub fn max_connections(&mut self, max_connections: u32) -> &mut Self {
        self.max_connections = max_connections;
        self
    }

    pub fn min_connections(&mut self, min_connections: u32) -> &mut Self {
        self.min_connections = min_connections;
        self
    }

    pub fn acquire_timeout(&mut self, acquire_timeout: Duration) -> &mut Self {
        self.acquire_timeout = acquire_timeout;
        self
    }

    pub fn acquire_slow_threshold(&mut self, acquire_slow_threshold: Duration) -> &mut Self {
        self.acquire_slow_threshold = acquire_slow_threshold;
        self
    }

    pub fn acquire_slow_level(&mut self, acquire_slow_level: LevelFilter) -> &mut Self {
        self.acquire_slow_level = acquire_slow_level;
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DataBase {
    #[cfg(feature = "mysql")]
    MySql(Pool<MySql>),

    #[cfg(feature = "postgres")]
    Postgres(Pool<Postgres>),

    #[cfg(feature = "sqlite")]
    Sqlite(Pool<Sqlite>),
}

impl DataBase {
    pub(crate) fn connect(options: &Options) -> Result<Self, SqlxError> {
        match options.connect_options {
            #[cfg(feature = "mysql")]
            ConnectionOptions::MySql(ref mysql_options) => {
                let mut pool_options = PoolOptions::<MySql>::new();
                let pool = pool_options
                    .max_connections(options.max_connections)
                    .min_connections(options.min_connections)
                    .acquire_timeout(options.acquire_timeout)
                    .acquire_slow_level(options.acquire_slow_level)
                    .acquire_slow_threshold(options.acquire_slow_threshold)
                    .after_connect(|connection, meta| Box::pin(async move {
                        event!(Level::INFO, "MySql after connect");

                        Ok(())
                    }))
                    .connect_lazy_with(mysql_options.clone());
                Ok(DataBase::MySql(pool))
            }
            #[cfg(feature = "postgres")]
            ConnectionOptions::Postgres(ref postgres_options) => {
                let mut pool_options = PoolOptions::<Postgres>::new();
                let pool = pool_options
                    .max_connections(options.max_connections)
                    .min_connections(options.min_connections)
                    .acquire_timeout(options.acquire_timeout)
                    .acquire_slow_level(options.acquire_slow_level)
                    .acquire_slow_threshold(options.acquire_slow_threshold)
                    .after_connect(|connection, meta| Box::pin(async move {
                        event!(Level::INFO, "Postgres after connect");

                        Ok(())
                    }))
                    .connect_lazy_with(postgres_options.clone());
                Ok(DataBase::Postgres(pool))
            }
            #[cfg(feature = "sqlite")]
            ConnectionOptions::Sqlite(ref sqlite_options) => {
                let mut pool_options = PoolOptions::<Sqlite>::new();
                let pool = pool_options
                    .max_connections(options.max_connections)
                    .min_connections(options.min_connections)
                    .acquire_timeout(options.acquire_timeout)
                    .acquire_slow_level(options.acquire_slow_level)
                    .acquire_slow_threshold(options.acquire_slow_threshold)
                    .after_connect(|connection, meta| Box::pin(async move {
                        event!(Level::INFO, "Sqlite after connect");

                        Ok(())
                    }))
                    .connect_lazy_with(sqlite_options.clone());
                Ok(DataBase::Sqlite(pool))
            }
        }
    }
}
