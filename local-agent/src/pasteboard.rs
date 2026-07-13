//! pasteboard — macOS NSPasteboard changeCount 检测
//!
//! 提供轻量级的剪贴板变更检测，无需复制粘贴板数据。
//! 非 macOS 平台返回 None（回退到定时轮询+内容哈希）。

#[cfg(target_os = "macos")]
mod imp {
    use objc2::rc::Retained;
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::NSInteger;

    pub fn change_count() -> i64 {
        unsafe {
            let pb = NSPasteboard::generalPasteboard();
            pb.changeCount() as i64
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    pub fn change_count() -> i64 {
        0
    }
}

/// 返回当前 NSPasteboard 的 changeCount。
/// 非 macOS 平台始终返回 `None`。
pub fn change_count() -> Option<i64> {
    #[cfg(target_os = "macos")]
    {
        Some(imp::change_count())
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}
