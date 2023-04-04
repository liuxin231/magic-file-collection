use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct Settings {
    pub log: Log,
    pub notify: Notify,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct Log {
    pub directory: String,
    pub file_name_prefix: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct Notify {
    pub watching_path: Vec<String>,
    pub file_name_regex: Vec<String>,
    pub flush_timing: Option<u64>,
    pub offset_file: Option<String>,
}
