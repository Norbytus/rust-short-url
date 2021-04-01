use std::fmt::Display;
use std::fmt::Write;

use actix_web::{HttpResponse, Responder, ResponseError, http::StatusCode};

pub mod file;

pub mod redis;

pub trait Storage<I> {
    fn find_short_url(&mut self, hash: String) -> E<Option<I>>;

    fn save_short_url<T: Into<I>>(&mut self, short_url: T) -> E<String>;
}

#[derive(Debug)]
pub struct ShortUrlStorageError {
    pub error: StorageError,
}

impl ShortUrlStorageError {
    pub fn undefined_error() -> Self {
        Self { error: StorageError::UndefinedError }
    }

    pub fn storage_temporarily_unavailable() -> Self {
        Self { error: StorageError::TemporarilyUnavailable }
    }

    pub fn error_on_save() -> Self {
        Self { error: StorageError::ErrorOnSave }
    }

    pub fn not_found() -> Self {
        Self { error: StorageError::NotFound }
    }
}

impl ResponseError for ShortUrlStorageError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.error {
            StorageError::NotFound => StatusCode::NOT_FOUND,
            StorageError::ErrorOnSave | StorageError::UndefinedError => StatusCode::INTERNAL_SERVER_ERROR,
            StorageError::TemporarilyUnavailable => StatusCode::LOCKED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let mut resp = HttpResponse::new(self.status_code());
        let mut buf = actix_web::web::BytesMut::new();
        let _ = write!(&mut buf, "{}", self);
        resp.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        resp.set_body(actix_web::dev::Body::from(buf))
    }
}

impl Display for ShortUrlStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error {
            StorageError::UndefinedError => write!(f, "Undefined error"),
            StorageError::TemporarilyUnavailable => write!(f, "Temporarily Unavailable"),
            StorageError::ErrorOnSave => write!(f, "Error on save"),
            StorageError::NotFound => write!(f, "Not found"),
        }
    }
}

#[derive(Debug)]
pub enum StorageError {
    UndefinedError,
    TemporarilyUnavailable,
    ErrorOnSave,
    NotFound,
}

impl Display for StorageError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub type E<T> = Result<T, ShortUrlStorageError>;
