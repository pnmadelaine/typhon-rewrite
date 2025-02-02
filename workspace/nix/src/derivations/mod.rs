mod types;

pub use types::*;

pub use crate::Error;

pub async fn derivation(drv: &str) -> Result<RecursiveDerivation, Error> {
    let mut cmd = tokio::process::Command::new("nix");
    cmd.args(["derivation", "show", "-r", drv]);
    let res = cmd.output().await.map_err(|_| Error::ToDo)?.stdout;
    serde_json::from_slice(&res).map_err(|_| Error::ToDo)
}
