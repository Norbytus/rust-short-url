use log::info;
use redis::{Client, Commands};

use crate::ShortUrlData;

use super::{ShortUrlStorageError, Storage};

pub struct RedisShortUrl {
    client: Client,
}

impl RedisShortUrl {
    pub fn new(client: Client) -> Self { Self { client } }
}

impl Storage<ShortUrlData> for RedisShortUrl {
    fn find_short_url(&mut self, hash: String) -> super::E<Option<ShortUrlData>> {
        let value: Result<Option<String>, _> = self.client.get(hash);
        if let Ok(value) = value {
            if let Some(value) = value {
                if let Ok(data) = serde_json::from_str::<ShortUrlData>(&value) {
                    Ok(Some(data))
                } else {
                    Err(ShortUrlStorageError::undefined_error())
                }
            } else {
                Ok(None)
            }

        } else {
            Err(ShortUrlStorageError::undefined_error())
        }
    }

    fn save_short_url<T: Into<ShortUrlData>>(&mut self, short_url: T) -> super::E<String> {
        let short_url: ShortUrlData = short_url.into();
        let raw = serde_json::to_string(&short_url).unwrap_or(String::new());

        let data = if let Some(ttl) = short_url.ttl {
            self.client.set_ex::<&str, String, ()>(&short_url.hash, raw, ttl as usize)
        } else {
            self.client.set::<&str, String, ()>(&short_url.hash, raw)
        };

        if let Err(e) = data {
            info!("{:?}", e);
            Err(ShortUrlStorageError::error_on_save())
        } else {
            Ok(short_url.hash)
        }
    }
}
