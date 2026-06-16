/// 共享 Tokio runtime（FRB 不在 Tokio runtime 中，需要手动创建）
use std::sync::OnceLock;

static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn shared_rt() -> &'static tokio::runtime::Runtime {
    RT.get_or_init(|| tokio::runtime::Runtime::new().expect("创建 Tokio runtime 失败"))
}
