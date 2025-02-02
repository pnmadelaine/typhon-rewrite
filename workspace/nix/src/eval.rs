use serde::de::DeserializeOwned;
use tokio::process::Command;

use crate::Error;

pub async fn eval<T: DeserializeOwned>(expr: &str) -> Result<T, Error> {
    Ok(cmd("nix-instantiate", &["--eval", "--expr", expr]).await?)
}

async fn cmd<T: DeserializeOwned>(cmd: &str, args: &[&str]) -> Result<T, Error> {
    let mut cmd = Command::new(cmd);
    cmd.args(args);
    let output = cmd.output().await.map_err(|_| Error::NixNotFound)?;
    if !output.status.success() {
        return Err(Error::Whatever(from_utf8(output.stderr)?));
    }
    let stdout = from_utf8(output.stdout)?;
    Ok(serde_json::from_str(&stdout)?)
}

fn from_utf8(bytes: Vec<u8>) -> Result<String, Error> {
    Ok(std::str::from_utf8(&bytes)
        .map(|s| s.to_owned())
        .map_err(|_| Error::NotUtf8(bytes))?)
}
