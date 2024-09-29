mod status;

pub(crate) use crate::utils::status::Status;
use sqlx::{Database, Error as SqlxError, Executor};

pub trait Ext<'c>: Executor<'c> {
    async fn get_status(self) -> Result<Status, SqlxError>;
}
