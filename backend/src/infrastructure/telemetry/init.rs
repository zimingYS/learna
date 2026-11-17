//! tracing 初始化，RUST_LOG 控制过滤

use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|err| {
        eprintln!("RUST_LOG 缺失或非法({err})，已回退到 info");
        EnvFilter::new("info")
    });

    let is_prod = matches!(std::env::var("APP_ENV").as_deref(), Ok("prod"));

    let result = if is_prod {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .try_init()
    } else {
        tracing_subscriber::fmt()
            .pretty()
            .with_target(true)
            .with_level(true)
            .with_env_filter(filter)
            .try_init()
    };

    if let Err(err) = result {
        eprintln!("tracing 初始化失败(可能已注册)：{err}");
    }
}
