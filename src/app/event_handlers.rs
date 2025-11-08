use super::AppState;
use crate::App;
use crate::core::question_type::{AdditionalCodeGenerator, Question, QuestionType};
use slint::ComponentHandle;
use std::str::FromStr;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// UI 事件处理器
pub struct EventHandlers {
    app_state: Arc<AppState>,
    stop_signal: Arc<AtomicBool>,
}

impl EventHandlers {
    /// 创建新的事件处理器
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { 
            app_state,
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 设置所有回调函数
    pub fn setup_callbacks(&self, app: &App) {
        self.setup_send_message_callback(app);
        self.setup_copy_callback(app);
        self.setup_stop_response_callback(app);
        self.setup_clear_image_callback(app);
    }

    /// 设置发送消息回调
    fn setup_send_message_callback(&self, app: &App) {
        let app_weak = app.as_weak();
        let current_image_path = self.app_state.current_image_path.clone();
        let llm_settings = self.app_state.llm_settings.clone();
        let stop_signal = self.stop_signal.clone();

        app.on_send_message(move || {
            let app_handle = app_weak.clone();
            let image_path_handle = current_image_path.clone();
            let llm_settings_handle = llm_settings.clone();
            let stop_signal_handle = stop_signal.clone();
            
            // 重置停止信号
            stop_signal_handle.store(false, Ordering::Relaxed);
            
            tracing::info!("[event_handlers] Send message triggered");

            // 获取当前文本
            if let Some(app) = app_handle.upgrade() {
                let text = app.get_prefill_text().to_string();
                if text.trim().is_empty() {
                    tracing::debug!("[event_handlers] No text to send");
                    return;
                }

                tracing::info!(
                    "[event_handlers] Sending message to LLM, text length: {}",
                    text.len()
                );

                // 设置流式状态
                app.set_is_streaming(true);
                app.set_model_reply("".into());

                // 获取当前图片路径
                let image_path = if let Ok(path_lock) = image_path_handle.lock() {
                    path_lock.as_ref().map(|p| p.clone())
                } else {
                    tracing::error!("[event_handlers] Failed to lock image path mutex");
                    None
                };

                if image_path.is_some() {
                    tracing::debug!("[event_handlers] Including image in LLM request");
                }
                tracing::info!("Question type: {}", app.get_question_type().as_str());
                let question = Question::new(
                    QuestionType::from_str(app.get_question_type().as_str())
                        .expect("wrong question type, please check again!{}"),
                    text,
                    image_path,
                );
                Self::handle_llm_request(app_handle, question, llm_settings_handle, stop_signal_handle);
            }
        });
    }

    /// 处理 LLM 请求
    fn handle_llm_request(
        app_handle: slint::Weak<App>,
        mut question: Question,
        llm_settings: Arc<std::sync::Mutex<crate::app::AppLLMSettingsManager>>,
        _stop_signal: Arc<AtomicBool>, // 保留参数以保持接口兼容，但当前实现不支持中途停止
    ) {
        tracing::info!("[event_handlers] Preparing to send LLM request");

        // 在后台线程中处理 LLM 请求
        let text_for_llm = question.prompt_stem();
        let image_path = question.img_path.clone();
        let app_for_response = app_handle.clone();
        
        tokio::spawn(async move {
            // 从设置中获取当前的 LLM manager
            let manager = if let Ok(settings) = llm_settings.lock() {
                crate::core::llm_backend::LLMManager::from_config(settings.get_config())
            } else {
                tracing::error!("[event_handlers] Failed to lock LLM settings, using default");
                crate::core::llm_backend::LLMManager::default()
            };

            let result = manager
                .send_message(text_for_llm, image_path.as_deref())
                .await;

            match result {
                Ok(response_content) => {
                    tracing::info!("[event_handlers] LLM request successful, response length: {}", response_content.len());
                    
                    // 使用 slint 的 invoke_from_event_loop 来确保 UI 更新在主线程中执行
                    let content = response_content.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_for_response.upgrade() {
                            tracing::debug!(
                                "[event_handlers] Updating UI with response length: {}",
                                content.len()
                            );
                            app.set_model_reply(content.into());
                            app.set_is_streaming(false);
                            tracing::info!("[event_handlers] LLM response completed, UI updated");
                        }
                    })
                    .ok();
                    
                    // 设置问题的回复
                    question.set_model_reply(response_content.into());
                }
                Err(e) => {
                    tracing::error!("[event_handlers] LLM request failed: {}", e);
                    
                    // 更新 UI 显示错误信息
                    let error_msg = format!("Error: {}", e);
                    slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_for_response.upgrade() {
                            app.set_model_reply(error_msg.into());
                            app.set_is_streaming(false);
                        }
                    })
                    .ok();
                }
            }
        });
    }

    /// 设置复制回调
    fn setup_copy_callback(&self, app: &App) {
        let app_weak = app.as_weak();

        app.on_copy_reply_and_addcode(move || {
            tracing::info!("[event_handlers] Copy reply triggered");
            if let Some(app) = app_weak.upgrade() {
                let reply = app.get_model_reply().to_string();
                if !reply.trim().is_empty() {
                    let additional_code = AdditionalCodeGenerator::new(
                        QuestionType::from_str(app.get_question_type().as_str())
                            .expect("wrong question type, please check again!"),
                    )
                    .get_code();
                    Self::copy_to_clipboard(&(reply + &additional_code));
                } else {
                    tracing::debug!("[event_handlers] No reply to copy");
                }
            }
        });
    }

    /// 设置停止响应回调
    fn setup_stop_response_callback(&self, app: &App) {
        let stop_signal = self.stop_signal.clone();
        let app_weak = app.as_weak();
        
        app.on_stop_response(move || {
            tracing::info!("[event_handlers] Stop response triggered");
            stop_signal.store(true, Ordering::Relaxed);
            
            // 立即更新 UI 状态，隐藏停止按钮
            if let Some(app) = app_weak.upgrade() {
                app.set_is_streaming(false);
                tracing::info!("[event_handlers] Streaming state set to false");
            }
        });
    }

    /// 设置清除图片回调
    fn setup_clear_image_callback(&self, app: &App) {
        let app_weak = app.as_weak();
        let current_image_path = self.app_state.current_image_path.clone();
        
        app.on_clear_image(move || {
            tracing::info!("[event_handlers] Clear image triggered");
            
            // 清除内存中的图片路径
            if let Ok(mut path) = current_image_path.lock() {
                *path = None;
            }
            
            // 清除UI中的图片
            if let Some(app) = app_weak.upgrade() {
                app.set_current_image(slint::Image::default());
                tracing::info!("[event_handlers] Image cleared successfully");
            }
        });
    }

    /// 复制文本到剪贴板
    fn copy_to_clipboard(text: &str) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Err(e) = clipboard.set_text(text) {
                tracing::error!("[event_handlers] Failed to copy to clipboard: {}", e);
            } else {
                tracing::info!("[event_handlers] Reply copied to clipboard successfully");
            }
        }
    }
}
