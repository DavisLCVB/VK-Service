#[derive(Debug)]
pub enum ApplicationError {
    NotFound,
    InternalError(String),
    DatabaseError(String),
    BadRequest(String),
    Unauthorized,
    PayloadTooLarge,
    InsufficientStorage,
    InvalidToken,
}
