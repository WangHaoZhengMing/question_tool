use crate::App;
use crate::app::AppLLMSettingsManager;
use crate::core::clipboard_monitor::start_clipboard_monitor;
use slint::ComponentHandle;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 应用全局状态管理
pub struct AppState {
    pub current_image_path: Arc<Mutex<Option<PathBuf>>>,
    pub clipboard_path: Arc<Mutex<Option<PathBuf>>>,
    pub llm_settings: Arc<Mutex<AppLLMSettingsManager>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("[app_state] Initializing application state");

        // 创建 LLM 设置管理器
        let llm_settings = match AppLLMSettingsManager::new() {
            Ok(settings) => {
                tracing::info!("[app_state] LLM settings manager initialized");
                settings
            }
            Err(e) => {
                tracing::error!("[app_state] Failed to initialize LLM settings: {}", e);
                return Err(e);
            }
        };


        Ok(Self {
            current_image_path: Arc::new(Mutex::new(None)),
            clipboard_path: Arc::new(Mutex::new(None)),
            llm_settings: Arc::new(Mutex::new(llm_settings)),
        })
    }

    pub fn setup_clipboard_monitor(&mut self) -> Arc<Mutex<Option<PathBuf>>> {
        tracing::info!("[app_state] Setting up clipboard monitor");
        let clipboard_path = start_clipboard_monitor();
        self.clipboard_path = clipboard_path.clone();
        clipboard_path
    }

    /// 设置所有 LLM 相关的回调函数
    pub fn setup_llm_callbacks(&self, app: &App) -> &AppState {
        self.setup_llm_provider_callback(app);
        self.setup_llm_model_callback(app);
        self.setup_llm_api_key_callback(app);
        self.setup_llm_base_url_callback(app);
        self.setup_llm_github_token_callback(app);
        self.setup_llm_streaming_callback(app);
        self.setup_llm_test_callback(app);
        self.setup_llm_save_callback(app);
        self.setup_llm_load_callback(app);
        &self
    }

    /// 初始化 UI 的 LLM 设置显示
    pub fn init_llm_ui_state(&self, app: &App) {
        if let Ok(settings) = self.llm_settings.lock() {
            let config = settings.get_config();

            app.set_llm_provider(config.provider.clone().into());
            app.set_llm_model(config.model.clone().into());
            app.set_llm_api_key(config.api_key.clone().unwrap_or_default().into());
            app.set_llm_base_url(config.base_url.clone().unwrap_or_default().into());
            app.set_llm_github_token(config.github_token.clone().unwrap_or_default().into());
            app.set_llm_enable_streaming(config.enable_streaming);

            tracing::info!(
                "[app_state] 初始化 LLM UI 状态: {}",
                settings.get_config_summary()
            );
        }
    }

    // LLM 提供商变更回调
    fn setup_llm_provider_callback(&self, app: &App) {
        let settings: Arc<Mutex<AppLLMSettingsManager>> = self.llm_settings.clone();
        app.on_llm_provider_changed(move |provider| {
            if let Ok(mut settings) = settings.lock() {
                settings.set_provider(provider.to_string());
            }
        });
    }

    // LLM 模型变更回调
    fn setup_llm_model_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        app.on_llm_model_changed(move |model| {
            if let Ok(mut settings) = settings.lock() {
                settings.set_model(model.to_string());
            }
        });
    }

    // LLM API Key 变更回调
    fn setup_llm_api_key_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        app.on_llm_api_key_changed(move |api_key| {
            if let Ok(mut settings) = settings.lock() {
                settings.set_api_key(api_key.to_string());
            }
        });
    }

    // LLM Base URL 变更回调
    fn setup_llm_base_url_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        app.on_llm_base_url_changed(move |base_url| {
            if let Ok(mut settings) = settings.lock() {
                settings.set_base_url(base_url.to_string());
            }
        });
    }

    // LLM GitHub Token 变更回调
    fn setup_llm_github_token_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        app.on_llm_github_token_changed(move |token| {
            if let Ok(mut settings) = settings.lock() {
                settings.set_github_token(token.to_string());
            }
        });
    }

    // LLM 流式设置变更回调
    fn setup_llm_streaming_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        app.on_llm_streaming_changed(move |enabled| {
            if let Ok(mut settings) = settings.lock() {
                settings.set_streaming(enabled);
            }
        });
    }

    // LLM 连接测试回调
    fn setup_llm_test_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        let app_weak = app.as_weak();
        app.on_llm_test_connection(move || {
            let settings = settings.clone();
            let app_weak = app_weak.clone();

            // 立即设置测试状态
            if let Some(app) = app_weak.upgrade() {
                app.set_llm_is_testing(true);
            }

            // 在后台线程中执行测试
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let result = runtime.block_on(async {
                    if let Ok(mut settings) = settings.lock() {
                        settings.test_connection().await
                    } else {
                        Err("无法访问设置".to_string())
                    }
                });

                let test_result = match result {
                    Ok(msg) => msg,
                    Err(msg) => msg,
                };

                // 使用 invoke_from_event_loop 更新 UI
                slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_weak.upgrade() {
                        app.set_llm_test_result(test_result.into());
                        app.set_llm_is_testing(false);
                    }
                })
                .ok();
            });
        });
    }

    // LLM 保存设置回调
    fn setup_llm_save_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        let app_weak = app.as_weak();
        app.on_llm_save_settings(move || {
            if let Ok(mut settings) = settings.lock() {
                match settings.save_config() {
                    Ok(_) => {
                        tracing::info!("[app_state] LLM 设置已保存");
                        if let Some(app) = app_weak.upgrade() {
                            app.set_llm_test_result("✅ 设置已保存".into());
                        }
                    }
                    Err(e) => {
                        tracing::error!("[app_state] 保存 LLM 设置失败: {}", e);
                        if let Some(app) = app_weak.upgrade() {
                            app.set_llm_test_result(format!("❌ 保存失败: {}", e).into());
                        }
                    }
                }
            }
        });
    }

    // LLM 加载设置回调
    fn setup_llm_load_callback(&self, app: &App) {
        let settings = self.llm_settings.clone();
        let app_weak = app.as_weak();
        app.on_llm_load_settings(move || {
            if let Ok(mut settings) = settings.lock() {
                match settings.reload_config() {
                    Ok(_) => {
                        tracing::info!("[app_state] LLM 设置已重新加载");

                        // 更新 UI 显示
                        if let Some(app) = app_weak.upgrade() {
                            let config = settings.get_config();
                            app.set_llm_provider(config.provider.clone().into());
                            app.set_llm_model(config.model.clone().into());
                            app.set_llm_api_key(config.api_key.clone().unwrap_or_default().into());
                            app.set_llm_base_url(
                                config.base_url.clone().unwrap_or_default().into(),
                            );
                            app.set_llm_github_token(
                                config.github_token.clone().unwrap_or_default().into(),
                            );
                            app.set_llm_enable_streaming(config.enable_streaming);
                            app.set_llm_test_result("✅ 设置已重新加载".into());
                        }
                    }
                    Err(e) => {
                        tracing::error!("[app_state] 重新加载 LLM 设置失败: {}", e);
                        if let Some(app) = app_weak.upgrade() {
                            app.set_llm_test_result(format!("❌ 加载失败: {}", e).into());
                        }
                    }
                }
            }
        });
    }
}
