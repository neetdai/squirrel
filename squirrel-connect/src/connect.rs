use std::ops::DerefMut;
use std::{ops::Deref, time::Duration};

use crate::utils::{Ext, Status};
use sqlx::any::{install_default_drivers, Any, AnyConnectOptions};
use sqlx::{
    pool::{Pool, PoolOptions},
    query, Connection, Database, Error as SqlxError,
};
use sqlx::{ConnectOptions, Execute};
use tracing::{event, log::LevelFilter, span, Level};
use url::{ParseError, Url};

#[cfg(feature = "mysql")]
use sqlx::mysql::{MySql, MySqlConnectOptions};

#[cfg(feature = "postgres")]
use sqlx::postgres::{PgConnectOptions, Postgres};

#[cfg(feature = "sqlite")]
use sqlx::sqlite::{Sqlite, SqliteConnectOptions};

#[derive(Debug, Clone)]
pub(crate) enum ConnectionOptions {
    #[cfg(feature = "mysql")]
    MySql(Url),

    #[cfg(feature = "postgres")]
    Postgres(Url),

    #[cfg(feature = "sqlite")]
    Sqlite(Url),
}

impl ConnectionOptions {
    pub(crate) fn from_url(url: &str) -> Result<Self, ParseError> {
        #[cfg(feature = "mysql")]
        if url.starts_with("mysql://") {
            return Ok(ConnectionOptions::MySql(Url::parse(url)?));
        }

        #[cfg(feature = "postgres")]
        if url.starts_with("postgres://") {
            return Ok(ConnectionOptions::Postgres(Url::parse(url)?));
        }

        #[cfg(feature = "sqlite")]
        if url.starts_with("sqlite://") {
            return Ok(ConnectionOptions::Sqlite(Url::parse(url)?));
        }

        Err(ParseError::EmptyHost)
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
    pub fn from_url(url: &str) -> Result<Self, ParseError> {
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
pub(crate) enum DataBaseType {
    #[cfg(feature = "mysql")]
    MySql,

    #[cfg(feature = "postgres")]
    Postgres,

    #[cfg(feature = "sqlite")]
    Sqlite,
}

#[derive(Debug, Clone)]
pub(crate) struct DataBase {
    database_type: DataBaseType,
    backend: Pool<Any>,
}

impl DataBase {
    pub(crate) fn connect(options: &Options) -> Result<Self, SqlxError> {
        install_default_drivers();
        let pool_option: PoolOptions<Any> = PoolOptions::new()
            .max_connections(options.max_connections)
            .min_connections(options.min_connections)
            .acquire_slow_level(options.acquire_slow_level)
            .acquire_slow_threshold(options.acquire_slow_threshold)
            .acquire_timeout(options.acquire_timeout);

        let (any_option, database_type) = match options.connect_options {
            #[cfg(feature = "mysql")]
            ConnectionOptions::MySql(ref url) => {
                (AnyConnectOptions::from_url(url)?, DataBaseType::MySql)
            }
            #[cfg(feature = "postgres")]
            ConnectionOptions::Postgres(ref url) => {
                (AnyConnectOptions::from_url(url)?, DataBaseType::Postgres)
            }
            #[cfg(feature = "sqlite")]
            ConnectionOptions::Sqlite(ref url) => {
                (AnyConnectOptions::from_url(url)?, DataBaseType::Sqlite)
            }
        };

        let pool = pool_option.connect_lazy_with(any_option);
        Ok(Self {
            backend: pool,
            database_type,
        })
    }

    pub(crate) async fn get_status(&self) -> Result<Status, SqlxError> {
        match self.database_type {
            #[cfg(feature = "mysql")]
            DataBaseType::MySql => {
                let result = query("show master status")
                    .fetch_optional(&self.backend)
                    .await?;
                if result.is_none() {
                    Ok(Status::Slave)
                } else {
                    Ok(Status::Master)
                }
            }
            #[cfg(feature = "postgres")]
            DataBaseType::Postgres => {
                let result = query("SELECT pg_is_in_recovery()")
                    .fetch_optional(&self.backend)
                    .await?;
                if result.is_none() {
                    Ok(Status::Master)
                } else {
                    Ok(Status::Slave)
                }
            }
            #[cfg(feature = "sqlite")]
            DataBaseType::Sqlite => Ok(Status::Master),
        }
    }
}

impl Deref for DataBase {
    type Target = Pool<Any>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl DerefMut for DataBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}
