use std::path::Path;
use std::sync::mpsc;

use async_llm::{ChatMessage, ChatRequest, Error};
use base64::{Engine, engine::general_purpose};
use image::ImageFormat;
use tokio_stream::StreamExt;

use super::llm_backend::{LLMBackend, LLMProvider, LLMResponse};

/// GPT 后端实现
#[derive(Clone, Debug)]
pub struct GPTBackend {
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for GPTBackend {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            api_key: None,
            base_url: None,
        }
    }
}

impl GPTBackend {
    #[allow(dead_code)]
    pub fn new(model: String) -> Self {
        Self {
            model,
            api_key: None,
            base_url: None,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// 从图片路径生成 base64 编码
    fn image_to_base64(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        // 打开并解码图片，然后编码为 PNG 格式字节流，最后转为 base64 字符串
        let image = image::ImageReader::open(path)?.decode()?;
        let mut buf = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)?;
        Ok(general_purpose::STANDARD.encode(&buf))
    }

    /// 构建消息列表
    fn build_messages(&self, text: &str, image_path: Option<&Path>) -> Vec<ChatMessage> {
        if let Some(path) = image_path {
            // 如果有图片，转换为 base64
            tracing::info!(
                "[gpt_backend] Converting image to base64: {}",
                path.display()
            );
            match self.image_to_base64(path) {
                Ok(base64) => {
                    tracing::info!("[gpt_backend] Image converted to base64 successfully");
                    let data_url = format!("data:image/png;base64,{}", base64);
                    vec![
                        ChatMessage::system(""),
                        ChatMessage::user_image_with_text(text, data_url.as_str()),
                    ]
                }
                Err(e) => {
                    tracing::error!("[gpt_backend] Failed to convert image to base64: {}", e);
                    vec![
                        ChatMessage::system(
                            "You are a helpful assistant for analyzing questions and images.",
                        ),
                        ChatMessage::user(text),
                    ]
                }
            }
        } else {
            // 只有文本
            tracing::info!("[gpt_backend] Text-only request");
            vec![
                ChatMessage::system(
                    "",
                ),
                ChatMessage::user(text),
            ]
        }
    }

    /// 尝试流式请求
    async fn try_streaming_request(
        &self,
        messages: Vec<ChatMessage>,
        response_sender: &mpsc::Sender<LLMResponse>,
    ) -> Result<String, Error> {
        tracing::info!("[gpt_backend] Attempting streaming request to GPT...");
        let stream_request = ChatRequest::new(&self.model, messages).with_stream();

        let mut response = stream_request.send_stream().await?;
        tracing::info!("[gpt_backend] Send streaming request successful, processing response...");

        let mut accumulated_content = String::new();

        while let Some(result) = response.next().await {
            match result {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        if let Some(delta) = &choice.delta {
                            if let Some(content) = &delta.content {
                                accumulated_content.push_str(content);

                                // 发送流式更新
                                tracing::trace!(
                                    "[gpt_backend] Streaming response chunk, total length: {}",
                                    accumulated_content.len()
                                );
                                let _ = response_sender.send(LLMResponse {
                                    content: accumulated_content.clone(),
                                    is_complete: false,
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("[gpt_backend] GPT streaming error during processing: {}", e);
                    return Err(e);
                }
            }
        }

        tracing::info!(
            "[gpt_backend] GPT streaming response completed, total length: {}",
            accumulated_content.len()
        );
        Ok(accumulated_content)
    }

    /// 尝试非流式请求
    async fn try_non_streaming_request(&self, messages: Vec<ChatMessage>) -> Result<String, Error> {
        tracing::info!("[gpt_backend] Attempting non-streaming request to GPT...");
        let request = ChatRequest::new(&self.model, messages);

        let response = request.send().await?;
        tracing::info!("[gpt_backend] Non-streaming request successful");

        let content = if let Some(choice) = response.choices.first() {
            if let Some(message) = &choice.message {
                message.content.clone().unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        tracing::info!(
            "[gpt_backend] GPT non-streaming response completed, length: {}",
            content.len()
        );
        Ok(content)
    }
}

#[async_trait::async_trait]
impl LLMBackend for GPTBackend {
    fn provider(&self) -> LLMProvider {
        LLMProvider::GPT
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    async fn send_message(
        &self,
        text: String,
        image_path: Option<&Path>,
        response_sender: mpsc::Sender<LLMResponse>,
    ) -> Result<(), Error> {
        let messages = self.build_messages(&text, image_path);

        // 首先尝试流式请求
        match self
            .try_streaming_request(messages.clone(), &response_sender)
            .await
        {
            Ok(content) => {
                // 流式请求成功完成
                let _ = response_sender.send(LLMResponse {
                    content,
                    is_complete: true,
                });
                Ok(())
            }
            Err(e) => {
                // 流式请求失败，尝试非流式请求
                tracing::warn!(
                    "[gpt_backend] Streaming request failed: {}, trying non-streaming request...",
                    e
                );

                match self.try_non_streaming_request(messages).await {
                    Ok(content) => {
                        // 发送完整响应
                        let _ = response_sender.send(LLMResponse {
                            content,
                            is_complete: true,
                        });
                        Ok(())
                    }
                    Err(e2) => {
                        tracing::error!(
                            "[gpt_backend] Both streaming and non-streaming requests failed. Streaming error: {}, Non-streaming error: {}",
                            e,
                            e2
                        );
                        let _ = response_sender.send(LLMResponse {
                            content: format!("Error: Both streaming and non-streaming requests failed. Last error: {}", e2),
                            is_complete: true,
                        });
                        Err(e2)
                    }
                }
            }
        }
    }

    async fn test_availability(&self) -> Result<String, Error> {
        tracing::info!("[gpt_backend] Testing GPT availability...");

        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user(
                "Please respond with 'Hello! I am working correctly.' to confirm you are available.",
            ),
        ];

        // 首先尝试流式请求
        tracing::info!("[gpt_backend] Attempting streaming test request...");
        let stream_request = ChatRequest::new(&self.model, messages.clone()).with_stream();

        match stream_request.send_stream().await {
            Ok(mut response) => {
                let mut accumulated_content = String::new();

                while let Some(result) = response.next().await {
                    match result {
                        Ok(response) => {
                            if let Some(choice) = response.choices.first() {
                                if let Some(delta) = &choice.delta {
                                    if let Some(content) = &delta.content {
                                        accumulated_content.push_str(content);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("[gpt_backend] GPT streaming test error: {}", e);
                            return Err(e);
                        }
                    }
                }

                if !accumulated_content.is_empty() {
                    tracing::info!(
                        "[gpt_backend] GPT streaming test successful: {}",
                        accumulated_content
                    );
                    Ok(accumulated_content)
                } else {
                    tracing::error!("[gpt_backend] GPT streaming test failed: No response content");
                    Err(Error::Stream("No response content from GPT".into()))
                }
            }
            Err(e) => {
                // 流式请求失败，尝试非流式请求
                tracing::warn!(
                    "[gpt_backend] Streaming test failed: {}, trying non-streaming test...",
                    e
                );

                let non_stream_request = ChatRequest::new(&self.model, messages);

                match non_stream_request.send().await {
                    Ok(response) => {
                        let content = if let Some(choice) = response.choices.first() {
                            if let Some(message) = &choice.message {
                                message.content.clone().unwrap_or_default()
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

                        if !content.is_empty() {
                            tracing::info!(
                                "[gpt_backend] GPT non-streaming test successful: {}",
                                content
                            );
                            Ok(content)
                        } else {
                            tracing::error!(
                                "[gpt_backend] GPT non-streaming test failed: No response content"
                            );
                            Err(Error::Stream("No response content from GPT".into()))
                        }
                    }
                    Err(e2) => {
                        tracing::error!(
                            "[gpt_backend] Both streaming and non-streaming tests failed. Streaming error: {}, Non-streaming error: {}",
                            e,
                            e2
                        );
                        Err(e2)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_environment() {
        dotenvy::dotenv().ok();

        // 设置环境变量，使用你的自定义 API 端点
        unsafe {
            if let (Ok(api_key), Ok(base_url)) = (
                std::env::var("OPENROUTER_API_KEY"),
                std::env::var("OPENROUTER_BASE_URL"),
            ) {
                std::env::set_var("OPENAI_API_KEY", api_key);
                std::env::set_var("OPENAI_BASE_URL", base_url);
            } else if let (Ok(api_key), Ok(base_url)) = (
                std::env::var("OPENAI_API_KEY"),
                std::env::var("OPENAI_BASE_URL"),
            ) {
                // 如果直接配置了 OPENAI_* 变量，则使用它们
                std::env::set_var("OPENAI_API_KEY", api_key);
                std::env::set_var("OPENAI_BASE_URL", base_url);
            }
        }

        println!("🔧 测试环境配置:");
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            println!("   Base URL: {}", base_url);
        }
        if let Ok(_) = std::env::var("OPENAI_API_KEY") {
            println!("   API Key: [已配置]");
        }
    }

    #[tokio::test]
    async fn test_gpt_connection() {
        // 初始化环境变量和日志
        setup_test_environment();
        let _ = tracing_subscriber::fmt::try_init();

        let backend = GPTBackend::default();

        match backend.test_availability().await {
            Ok(response) => {
                println!("✅ GPT 可用! 响应: {}", response);
                assert!(!response.is_empty(), "GPT response should not be empty");
            }
            Err(e) => {
                println!("❌ GPT 不可用: {}", e);
                eprintln!(
                    "GPT test failed (this might be expected if no API key is configured): {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_send_message_to_gpt() {
        setup_test_environment();
        let _ = tracing_subscriber::fmt::try_init();

        let backend = GPTBackend::default();
        let (sender, receiver) = mpsc::channel();

        let test_message = "Hello, this is a test message.".to_string();

        // 启动异步任务发送消息
        let send_task =
            tokio::spawn(async move { backend.send_message(test_message, None, sender).await });

        // 收集响应
        let mut responses = Vec::new();
        let mut final_content = String::new();

        // 设置超时以避免测试无限等待
        let timeout_duration = std::time::Duration::from_secs(30);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout_duration {
            match receiver.try_recv() {
                Ok(response) => {
                    responses.push(response.clone());
                    final_content = response.content.clone();

                    if response.is_complete {
                        break;
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }

        // 等待发送任务完成
        match send_task.await {
            Ok(Ok(())) => {
                println!("✅ 消息发送成功!");
                println!("📝 最终响应长度: {}", final_content.len());
                println!("📊 总共收到 {} 个响应片段", responses.len());

                if !final_content.is_empty() {
                    println!(
                        "📄 响应内容预览: {}...",
                        final_content.chars().take(100).collect::<String>()
                    );
                }
            }
            Ok(Err(e)) => {
                println!("❌ GPT 请求失败: {}", e);
                eprintln!(
                    "Send message test failed (this might be expected if no API key is configured): {}",
                    e
                );
            }
            Err(e) => {
                println!("❌ 任务执行失败: {}", e);
            }
        }
    }
}
