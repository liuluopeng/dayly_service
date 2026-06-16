use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use std::sync::RwLock;
static LOGGER: RwLock<Option<StreamSink<String>>> = RwLock::new(None);

#[flutter_rust_bridge::frb(sync)]
pub fn init_rust_logger(sink: StreamSink<String>) {
    let mut logger: std::sync::RwLockWriteGuard<Option<StreamSink<String>>> = match LOGGER.write() {
        Ok(val) => val,
        Err(val) => val.into_inner(),
    };
    *logger = Some(sink);
}

#[flutter_rust_bridge::frb(ignore)]
pub fn log_to_dart(msg: String) {
    let logger: std::sync::RwLockReadGuard<Option<StreamSink<String>>> = match LOGGER.read() {
        Ok(val) => val,
        Err(val) => val.into_inner(),
    };
    if let Some(logger) = logger.as_ref() {
        let _ = logger.add(msg);
    }
}
