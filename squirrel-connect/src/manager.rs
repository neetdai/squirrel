use std::collections::VecDeque;

use sqlx::{
    pool::{Pool, PoolOptions},
    Error as SqlxError,
};

use crate::{connect::DataBase, Options};

#[derive(Debug)]
pub struct Manager {
    master: VecDeque<DataBase>,
    slaves: VecDeque<DataBase>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            master: VecDeque::new(),
            slaves: VecDeque::new(),
        }
    }

    pub fn add_master(&mut self, options: &Options) -> Result<(), SqlxError> {
        let database = DataBase::connect(options)?;
        self.master.push_back(database);

        Ok(())
    }
}
