//! 生成 zici.db（宿主机预生成，词典同款 volume 挂载方案）
//! 用法: cargo run -p lx_dayly_service --bin gen_zici_db [输出路径，默认 cold_data/zici.db]

fn main() {
    let target = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "cold_data/zici.db".to_string());
    match lx_dayly_service::zici_db::generate_db(&target) {
        Ok(()) => println!("zici.db 已生成: {}", target),
        Err(e) => {
            eprintln!("生成失败: {}", e);
            std::process::exit(1);
        }
    }
}
