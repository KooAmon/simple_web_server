use std::{
    time::{SystemTime, UNIX_EPOCH},
     fmt::{Display, Formatter}
};

use uuid::Uuid;

#[allow(dead_code)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Trace,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "Error"),
            Self::Warning => write!(f, "Warning"),
            Self::Info => write!(f, "Info"),
            Self::Trace => write!(f, "Trace"),
        }
    }
}


pub fn log(level: LogLevel, serverid: &Uuid, sessionid: &Uuid, message: &str) {
    //  UTC time, server id, session id, message
    println!("{},{},{},{},{}", unix_ms(), level, serverid, sessionid, message);
}

/// Returns the current unix timestamp in milliseconds.
#[inline]
fn unix_ms() -> u64 {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch");
    ts.as_millis() as u64
}