// 自定义console_log宏，类似println!语法
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format_args!($($t)*).to_string().into());
    }
}
