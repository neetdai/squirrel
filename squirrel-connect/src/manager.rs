use std::collections::VecDeque;

use sqlx::pool::{Pool, PoolOptions};

use crate::connect::DataBase;

#[derive(Debug)]
pub struct Manager {
    master: VecDeque<DataBase>,
    slaves: VecDeque<DataBase>,
}

impl Manager {}
