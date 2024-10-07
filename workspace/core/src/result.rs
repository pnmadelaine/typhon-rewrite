pub type InternalResult<T> = std::result::Result<T, InternalError>;

#[derive(Debug)]
pub enum InternalError {
    UnexpectedDatabaseError(diesel::result::Error),
    UnexpectedPoolError(diesel_async::pooled_connection::bb8::RunError),
    UnexpectedRuntimeError,
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let err = match self {
            Self::UnexpectedDatabaseError(err) => format!("Unexpected database error: {err:?}"),
            Self::UnexpectedPoolError(err) => format!("Unexpected database error: {err:?}"),
            Self::UnexpectedRuntimeError => format!("Unexpected runtime error"),
        };
        write!(f, "{err}")
    }
}

impl From<diesel::result::Error> for InternalError {
    fn from(err: diesel::result::Error) -> Self {
        Self::UnexpectedDatabaseError(err)
    }
}

impl From<diesel_async::pooled_connection::PoolError> for InternalError {
    fn from(err: diesel_async::pooled_connection::PoolError) -> Self {
        Self::UnexpectedPoolError(diesel_async::pooled_connection::bb8::RunError::User(err))
    }
}

impl From<diesel_async::pooled_connection::bb8::RunError> for InternalError {
    fn from(err: diesel_async::pooled_connection::bb8::RunError) -> Self {
        Self::UnexpectedPoolError(err)
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for InternalError {
    fn from(_err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::UnexpectedRuntimeError
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for InternalError {
    fn from(_err: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::UnexpectedRuntimeError
    }
}

impl From<tokio::task::JoinError> for InternalError {
    fn from(_err: tokio::task::JoinError) -> Self {
        Self::UnexpectedRuntimeError
    }
}
