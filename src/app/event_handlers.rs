use super::AppState;
use crate::App;
use crate::core::llm_backend::{LLMResponse,send_message_to_llm};
use crate::core::question_type::{AdditionalCodeGenerator, Question, QuestionType};
use slint::ComponentHandle;
use std::str::FromStr;
use std::sync::{Arc, mpsc};

/// UI 事件处理器
pub struct EventHandlers {
    app_state: Arc<AppState>,
}

impl EventHandlers {
    /// 创建新的事件处理器
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// 设置所有回调函数
    pub fn setup_callbacks(&self, app: &App) {
        self.setup_send_message_callback(app);
        self.setup_copy_callback(app);
    }

    /// 设置发送消息回调
    fn setup_send_message_callback(&self, app: &App) {
        let app_weak = app.as_weak();
        let current_image_path = self.app_state.current_image_path.clone();

        app.on_send_message(move || {
            let app_handle = app_weak.clone();
            let image_path_handle = current_image_path.clone();
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
                Self::handle_llm_request(app_handle, question);
            }
        });
    }

    /// 处理 LLM 请求
    fn handle_llm_request(app_handle: slint::Weak<App>, mut question: Question) {
        // 创建响应通道
        tracing::info!("[event_handlers] Preparing to send LLM request");
        let (response_sender, response_receiver) = mpsc::channel::<LLMResponse>();

        // 在后台线程中处理 LLM 请求
        let text_for_llm = question.prompt_stem();
        let image_path = question.img_path.clone();
        tokio::spawn(async move {
            let result =
                send_message_to_llm(text_for_llm, image_path.as_deref(), response_sender).await;

            if let Err(e) = result {
                tracing::error!("[event_handlers] LLM request failed: {}", e);
            }
        });

        // 在主线程中处理响应
        let app_for_response = app_handle.clone();
        std::thread::spawn(move || {
            while let Ok(response) = response_receiver.recv() {
                tracing::trace!("[event_handlers] Received LLM response chunk, length: {}", response.content.len());
                
                // 使用slint的invoke_from_event_loop来确保UI更新在主线程中执行
                let content = response.content.clone();
                let is_complete = response.is_complete;
                let app_weak = app_for_response.clone();
                
                slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_weak.upgrade() {
                        tracing::debug!("[event_handlers] Updating UI with response length: {}", content.len());
                        app.set_model_reply(content.into());
                        
                        if is_complete {
                            app.set_is_streaming(false);
                            tracing::info!("[event_handlers] LLM response completed, UI updated");
                        }
                    }
                }).ok();

                if is_complete {
                    question.set_model_reply(response.content.into());
                    break;
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
                        QuestionType::from_str(
                            app.get_question_type().as_str()
                        )
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
