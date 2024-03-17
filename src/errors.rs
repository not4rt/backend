use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, From};
use sqlx::Error as PoolError;

#[derive(Display, From, Debug)]
pub enum MyError {
    NotFound,
    Unprocessable,
    PoolError(PoolError),
}
impl std::error::Error for MyError {}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            MyError::NotFound => HttpResponse::NotFound().finish(),
            MyError::Unprocessable => HttpResponse::UnprocessableEntity().finish(),
            MyError::PoolError(ref err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
            //_ => HttpResponse::InternalServerError().finish(),
        }
    }
}

