use hyper::StatusCode;
use tracing::error;

pub fn internal_error<T: std::fmt::Debug>(error: T) -> StatusCode {
    error!("Internal error: {error:?}");
    StatusCode::INTERNAL_SERVER_ERROR
}
