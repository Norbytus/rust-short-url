use super::*;
use std::io::{BufWriter, BufReader, Write, BufRead};
use crate::ShortUrlData;
use std::fs::File;

#[derive(Debug)]
pub struct FileStorage {
    pub file: File,
}

impl Storage<ShortUrlData> for FileStorage {
    fn find_short_url(&mut self, hash: String) -> E<Option<ShortUrlData>> {
        let buffer = BufReader::new(&mut self.file);

        let data: Vec<ShortUrlData> = buffer
            .lines()
            .map(|line| line.unwrap_or(String::new()))
            .map(|str_data| serde_json::from_str::<ShortUrlData>(&str_data))
            .filter(|serd_result| serd_result.is_ok())
            .map(|serd_result| serd_result.unwrap())
            .filter(move |short_url| short_url.hash == hash)
            .collect();

        Ok(data.get(0).map(|v| v.clone()))
    }

    fn save_short_url<T: Into<ShortUrlData>>(&mut self, data: T) -> E<String> {
        let short_url: ShortUrlData = data.into();
        let raw = serde_json::to_string(&short_url).unwrap_or(String::new());

        let mut buff = BufWriter::new(&mut self.file);

        if let Err(e) = buff.write(raw.as_bytes()) {
            Err(ShortUrlStorageError {
                error: StorageError::ErrorOnSave,
            })
        } else {
            let _ = buff.write("\n".as_bytes());
            Ok(short_url.hash.clone())
        }
    }
}
