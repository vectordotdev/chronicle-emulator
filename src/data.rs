use chrono::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// Store the created entries.
pub static DATA: Lazy<Mutex<Vec<Log>>> = Lazy::new(|| Mutex::new(vec![]));

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Log {
    customer_id: String,
    pub log_type: String,
    log_text: String,
    ts_rfc3339: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UnstructuredLogs {
    customer_id: String,
    log_type: String,
    entries: Vec<UnstructuredLog>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UnstructuredLog {
    log_text: String,
    ts_epoch_microseconds: Option<i64>,
    ts_rfc3339: Option<String>,
}

impl From<UnstructuredLogs> for Vec<Log> {
    fn from(logs: UnstructuredLogs) -> Self {
        logs.entries
            .into_iter()
            .map(|entry| Log {
                customer_id: logs.customer_id.clone(),
                log_type: logs.log_type.clone(),
                log_text: entry.log_text,
                ts_rfc3339: entry.ts_rfc3339.unwrap_or_else(|| {
                    entry
                        .ts_epoch_microseconds
                        .map(|ms| Utc.timestamp_millis(ms))
                        .unwrap_or_else(Utc::now)
                        .to_rfc3339()
                }),
            })
            .collect()
    }
}

/// Adds the logs to our (in memory) database.
pub fn add_to_data(logs: UnstructuredLogs) {
    let mut data = DATA.lock().unwrap();
    data.append(&mut Vec::from(logs));
}
