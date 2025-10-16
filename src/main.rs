// 在 Windows Release 模式下隐藏控制台窗口
// #![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Ensure the generated Slint modules are included
slint::include_modules!();
use slint::ComponentHandle;

mod app;
mod core;

use crate::app::{AppState, ClipboardTimer, EventHandlers};
use crate::core::logger;

#[tokio::main]
async fn main() {
    // 初始化环境
    setup_environment();
    let app = App::new().unwrap();

    // 创建应用状态和组件
    let mut app_state = match AppState::new() {
        Ok(state) => {
            tracing::info!("[main] Application state initialized");
            state
        }
        Err(e) => {
            tracing::error!("[main] Failed to initialize application state: {}", e);
            panic!("Cannot create application state: {}", e);
        }
    };

    let _clipboard_path = app_state.setup_clipboard_monitor();
    let app_state = Arc::new(app_state);

    // 设置 LLM 相关回调和 UI 状态
    app_state.setup_llm_callbacks(&app).init_llm_ui_state(&app);

    let mut clipboard_timer = ClipboardTimer::new(app_state.clone());
    clipboard_timer.start(app.as_weak());
    EventHandlers::new(app_state.clone()).setup_callbacks(&app);

    tracing::info!("[main] Application initialized, starting UI loop");

    // 运行 UI 主循环
    app.run().unwrap();
}

/// 设置应用环境
fn setup_environment() {
    dotenvy::dotenv().ok();
    logger::init();
    tracing::info!("[main] Application starting");

    // 如果环境变量存在，设置 OpenAI 相关变量
    if let (Ok(api_key), Ok(base_url)) = (
        std::env::var("OPENROUTER_API_KEY"),
        std::env::var("OPENROUTER_BASE_URL"),
    ) {
        unsafe {
            std::env::set_var("OPENAI_API_KEY", api_key);
            std::env::set_var("OPENAI_BASE_URL", base_url);
        }
    }
}
