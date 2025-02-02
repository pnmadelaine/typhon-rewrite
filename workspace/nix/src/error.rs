use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    Whatever(String),
    NixNotFound,
    NetworkUnavailable(String),
    UnexpectedResponse(String),
    #[from]
    NixDaemon(crate::store::StoreError),
    #[from]
    Deserialization(serde_json::Error),
    #[from]
    Git(git2::Error),
    NotUtf8(Vec<u8>),
    #[from]
    Join(tokio::task::JoinError),
    ToDo,
}
