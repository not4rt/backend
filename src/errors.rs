use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, From};
use diesel::result::Error::{self as DbError, NotFound};

#[derive(Display, From, Debug)]
pub enum MyError {
    NotFound,
    Unprocessable,
    DbError(DbError)
    //PoolError(PoolError),
}
impl std::error::Error for MyError {}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            MyError::NotFound => HttpResponse::NotFound().finish(),
            MyError::Unprocessable => HttpResponse::UnprocessableEntity().finish(),
            MyError::DbError(ref err) => {
                if err == &NotFound {
                    return HttpResponse::UnprocessableEntity().finish()
                }
                HttpResponse::InternalServerError().body(err.to_string())
            },
            // MyError::PoolError(ref err) => {
            //     HttpResponse::InternalServerError().body(err.to_string())
            // }
            //_ => HttpResponse::InternalServerError().finish(),
        }
    }
}

