use actix_web::{HttpResponse, dev::HttpResponseBuilder, http::{StatusCode, header::Header}, web::Data};
use actix_web::{web::Json, Responder};
use actix_web::{get, post};
use actix_web::{App, HttpServer};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

mod storage;

use storage::{ShortUrlStorageError, Storage, StorageError};

#[actix_web::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    simple_logger::SimpleLogger::new().init()?;

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open("db.txt")
        .expect("Can't create file");

    let arc_file = Arc::new(Mutex::new(storage::file::FileStorage { file }));

    HttpServer::new(move || App::new().data(arc_file.clone()).service(create_short_url).service(redirect))
        .bind("127.0.0.1:8080")
        .expect("Port already used")
        .run()
        .await?;

    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
struct ShortUrlRequest {
    url: String,
    ttl: Option<u64>,
}

impl std::convert::Into<ShortUrlData> for ShortUrlRequest {
    fn into(self) -> ShortUrlData {
        ShortUrlData {
            source: self.url,
            hash: nanoid!(),
            ttl: self.ttl,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ShortUrlData {
    source: String,
    hash: String,
    ttl: Option<u64>,
}

#[post("/url")]
async fn create_short_url(
    data: Json<ShortUrlRequest>,
    storage: Data<Arc<Mutex<storage::file::FileStorage>>>,
) -> impl Responder {
    let mut storage = if let Ok(storage) = storage.try_lock() {
        storage
    } else {
        return Err(ShortUrlStorageError::storage_temporarily_unavailable());
    };

    storage.save_short_url(data.into_inner())
}

#[get("/{hash}")]
async fn redirect(
    hash: actix_web::web::Path<String>,
    storage: Data<Arc<Mutex<storage::file::FileStorage>>>,
) -> impl Responder {
    let mut storage = if let Ok(storage) = storage.try_lock() {
        storage
    } else {
        return Err(ShortUrlStorageError::storage_temporarily_unavailable());
    };

    let url = storage.find_short_url(hash.into_inner())?
        .ok_or_else(|| ShortUrlStorageError::not_found())?;

    Ok(HttpResponse::Found().set_header("Location", url.source).finish())
}
