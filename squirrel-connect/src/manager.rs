use std::collections::VecDeque;

use super::utils::{Ext, Status};
use sqlx::{
    pool::{Pool, PoolOptions},
    Error as SqlxError,
};

use crate::{connect::DataBase, Options};

#[derive(Debug)]
pub struct Manager {
    wait_queue: VecDeque<DataBase>,
    master: VecDeque<DataBase>,
    slaves: VecDeque<DataBase>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            wait_queue: VecDeque::new(),
            master: VecDeque::new(),
            slaves: VecDeque::new(),
        }
    }

    pub fn add_connect_options(&mut self, options: &Options) -> Result<(), SqlxError> {
        let database = DataBase::connect(options)?;
        self.wait_queue.push_back(database);

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), SqlxError> {
        for p in self.master.iter() {
            p.get_status().await?;
        }

        Ok(())
    }
}
