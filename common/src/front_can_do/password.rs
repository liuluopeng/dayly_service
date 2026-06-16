use rand::{Rng, seq::SliceRandom};

/// 生成随机密码
/// # Arguments
/// * `length` - 密码长度
/// # Returns
/// * 随机生成的密码字符串
pub fn generate_password(length: usize) -> String {
    // 英文键盘上的字符，包括大小写字母、数字和常见符号
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;':,.<>/?";

    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    password
}

/// 生成指定长度的随机密码，包含至少一个大写字母、一个小写字母、一个数字和一个特殊字符
/// # Arguments
/// * `length` - 密码长度，至少为 4
/// # Returns
/// * 随机生成的强密码字符串
pub fn generate_strong_password(length: usize) -> String {
    // 确保密码长度至少为 4
    let length = if length < 4 { 4 } else { length };

    // 定义不同类型的字符集
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const DIGITS: &[u8] = b"0123456789";
    const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;':,.<>/?";
    const ALL_CHARACTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;':,.<>/?";

    let mut rng = rand::thread_rng();
    let mut password = Vec::with_capacity(length);

    // 确保至少包含一个大写字母、一个小写字母、一个数字和一个特殊字符
    password.push(UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char);
    password.push(LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char);
    password.push(DIGITS[rng.gen_range(0..DIGITS.len())] as char);
    password.push(SYMBOLS[rng.gen_range(0..SYMBOLS.len())] as char);

    // 填充剩余长度
    for _ in 4..length {
        password.push(ALL_CHARACTERS[rng.gen_range(0..ALL_CHARACTERS.len())] as char);
    }

    // 打乱密码顺序
    password.shuffle(&mut rng);

    password.into_iter().collect()
}
