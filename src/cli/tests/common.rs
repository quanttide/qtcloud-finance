use std::fs;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// 创建临时 Beancount 文件，返回路径。每个调用获得唯一路径。
pub fn write_beancount(content: &str) -> String {
    let dir = std::env::temp_dir().join("qtcloud_finance_test");
    fs::create_dir_all(&dir).unwrap();

    let pid = std::process::id();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.join(format!("{}_{}_{}.beancount", pid, ts, n));

    let mut f = fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path.to_string_lossy().to_string()
}

/// 清理测试文件
pub fn cleanup(path: &str) {
    let _ = fs::remove_file(path);
}
