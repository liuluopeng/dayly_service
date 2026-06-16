/// 加法运算
#[flutter_rust_bridge::frb(sync)]
pub fn add(a: i32, b: i32) -> i32 {
    a + b + 10
}

/// 加法运算（+22）
#[flutter_rust_bridge::frb(sync)]
pub fn add22(a: i32, b: i32) -> i32 {
    a + b + 22
}

/// 乘法运算
#[flutter_rust_bridge::frb(sync)]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
