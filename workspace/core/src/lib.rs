#![allow(dead_code)]

mod database;
mod logs;
mod models;
mod permissions;
mod prelude;
mod result;
// mod tasks;
mod users;

pub mod builds;
pub mod init;
pub mod projects;

pub use logs::Read;
pub use typhon_nix::Source;

use crate::prelude::*;

pub struct Settings {
    pub database_url: String,
}

pub async fn repair_database(url: &str) {
    let pool = Pool::new(url);
    pool.get().await.repair().await;
}

pub async fn run_pending_migrations(url: &str) {
    let pool = Pool::new(url);
    pool.get_owned().await.run_pending_migrations().await;
}

pub fn typhon(settings: Settings) -> (init::Addr, JoinHandle<()>) {
    init::init(settings)
}
