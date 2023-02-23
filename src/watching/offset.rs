use crate::app_config::Settings;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref FILE_OFFSET: Arc<Mutex<HashMap<String, usize>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub async fn flush(settings: Arc<Settings>) {
    let mut offset_file_path_str = String::from("./data/offset");
    if settings.notify.offset_file.is_some() {
        offset_file_path_str = String::from(settings.notify.offset_file.as_ref().unwrap());
    }
    let offset_file_path = std::path::Path::new(&offset_file_path_str);
    if !offset_file_path.exists() {
        std::fs::File::create(&offset_file_path).unwrap();
    } else {
        let offset_file = std::fs::File::open(offset_file_path).unwrap();
        let mut reader = BufReader::new(offset_file);
        let mut file_content = String::new();
        reader.read_to_string(&mut file_content).unwrap();
        if let Ok(offset_map) = serde_json::from_str(&file_content) {
            let mut file_offset_map = FILE_OFFSET.lock().unwrap();
            *file_offset_map = offset_map;
        }
    }
    loop {
        let flush_timing = settings.notify.flush_timing.unwrap_or(10);
        tokio::time::sleep(tokio::time::Duration::from_secs(flush_timing)).await;
        let file_offset = FILE_OFFSET.lock().unwrap();
        let string = serde_json::to_string(&*file_offset).unwrap();
        let mut offset_file = std::fs::File::create(offset_file_path).unwrap();
        offset_file.write_all(string.as_bytes()).unwrap();
    }
}

pub fn get_offset_by_key(key: &String) -> usize {
    let mut file_offset = FILE_OFFSET.lock().unwrap();
    if file_offset.get(key.as_str()).is_none() {
        file_offset.insert(key.to_string(), 0);
        return 0;
    }
    let offset = file_offset.get(key.as_str()).unwrap();
    let offset = *offset;
    offset
}

pub fn set_offset(key: &String, offset: usize) {
    let mut file_offset = FILE_OFFSET.lock().unwrap();
    let now_offset = file_offset.get(key.as_str());
    if now_offset.is_some() {
        let now_offset = now_offset.unwrap();
        if offset <= *now_offset {
            return;
        }
    }
    file_offset.insert(key.to_string(), offset);
}
