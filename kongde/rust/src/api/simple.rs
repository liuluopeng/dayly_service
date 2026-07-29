#[flutter_rust_bridge::frb(sync)]
pub fn greet(name: String) -> String {
    tracing::info!(name = %name, "greet called");
    format!("我是RUST Hello, {name}!")
}
#[flutter_rust_bridge::frb(sync)]
pub fn greet2(name: String) -> String {
    tracing::info!(name = %name, "greet2 called");
    format!("我是RUST Hello, {name}!")
}
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
