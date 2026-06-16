use crate::api::logger_bridge::log_to_dart;

#[flutter_rust_bridge::frb(sync)]
pub fn greet(name: String) -> String {
    log_to_dart(format!("给dart的日志 greet called with name: {}", name));

    format!("我是RUST Hello, {name}!")
}
#[flutter_rust_bridge::frb(sync)]
pub fn greet2(name: String) -> String {
    log_to_dart(format!("给dart的日志 greet called with name: {}", name));

    format!("我是RUST Hello, {name}!")
}
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
