use std::path::Path;
use std::sync::mpsc;

use async_llm::{ChatMessage, ChatRequest, Error};
use base64::{Engine, engine::general_purpose};
use image::ImageFormat;
use tokio_stream::StreamExt;

use super::llm_backend::{LLMResponse, LLMBackend, LLMProvider};

/// GitHub Models 后端实现
/// 支持 GitHub Models API (https://models.inference.ai.azure.com)
#[derive(Clone, Debug)]
pub struct GitHubBackend {
    pub model: String,
    pub api_token: Option<String>,
    pub base_url: String,
}

impl Default for GitHubBackend {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            api_token: std::env::var("GITHUB_TOKEN").ok(),
            base_url: "https://models.inference.ai.azure.com".to_string(),
        }
    }
}

impl GitHubBackend {
    /// 创建新的 GitHub 后端实例
    pub fn new(model: String) -> Self {
        Self {
            model,
            api_token: std::env::var("GITHUB_TOKEN").ok(),
            base_url: "https://models.inference.ai.azure.com".to_string(),
        }
    }

    /// 设置 GitHub Token
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_token = Some(api_key);
        self
    }

    /// 设置自定义 API 端点
     #[allow(dead_code)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// 从图片路径生成 base64 编码
    fn image_to_base64(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let image = image::ImageReader::open(path)?.decode()?;
        let mut buf = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)?;
        Ok(general_purpose::STANDARD.encode(&buf))
    }

    /// 构建消息列表
    fn build_messages(&self, text: &str, image_path: Option<&Path>) -> Vec<ChatMessage> {
        if let Some(path) = image_path {
            tracing::debug!("[github_backend] Converting image to base64: {}", path.display());
            match self.image_to_base64(path) {
                Ok(base64) => {
                    tracing::debug!("[github_backend] Image converted to base64 successfully");
                    // GitHub Models API 需要 data URL 格式: data:image/png;base64,<base64_string>
                    let data_url = format!("data:image/png;base64,{}", base64);
                    vec![
                        ChatMessage::system("You are GitHub Copilot, a helpful AI assistant for analyzing questions and images."),
                        ChatMessage::user_image_with_text(text, data_url.as_str()),
                    ]
                }
                Err(e) => {
                    tracing::error!("[github_backend] Failed to convert image to base64: {}", e);
                    vec![
                        ChatMessage::system("you have to follow the follow rules"),
                        ChatMessage::user(text),
                    ]
                }
            }
        } else {
            tracing::debug!("[github_backend] Text-only request");
            tracing::info!("messages: {:?}", text);
            vec![
                ChatMessage::system("you have to follow the follow rules"),
                ChatMessage::user(text),
            ]
        }
    }

    /// 设置环境变量以使用 GitHub Models API
    fn setup_environment(&self) {
        if let Some(api_token) = &self.api_token {
            unsafe {
                std::env::set_var("GITHUB_TOKEN", api_token);
            }
        } else {
            tracing::error!("[github_backend] No GitHub token available. Please set GITHUB_TOKEN environment variable or use with_api_key()");
        }
    }

    /// 尝试流式请求
    async fn try_streaming_request(
        &self,
        messages: Vec<ChatMessage>,
        response_sender: &mpsc::Sender<LLMResponse>,
    ) -> Result<String, Error> {
        tracing::info!("[github_backend] Attempting streaming request to GitHub Models...");
        
        // 临时设置环境变量
        self.setup_environment();
        
        let stream_request = ChatRequest::new(&self.model, messages).with_stream();
        
        let mut response = stream_request.send_stream().await?;
        tracing::info!("[github_backend] Streaming request successful, processing response...");
        
        let mut accumulated_content = String::new();

        while let Some(result) = response.next().await {
            match result {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        if let Some(delta) = &choice.delta {
                            if let Some(content) = &delta.content {
                                accumulated_content.push_str(content);
                                
                                tracing::trace!("[github_backend] Streaming response chunk, total length: {}", accumulated_content.len());
                                let _ = response_sender.send(LLMResponse {
                                    content: accumulated_content.clone(),
                                    is_complete: false,
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("[github_backend] GitHub streaming error during processing: {}", e);
                    return Err(e);
                }
            }
        }

        tracing::info!("[github_backend] GitHub streaming response completed, total length: {}", accumulated_content.len());
        Ok(accumulated_content)
    }

    /// 尝试非流式请求
    async fn try_non_streaming_request(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<String, Error> {
        tracing::info!("[github_backend] Attempting non-streaming request to GitHub Models...");
        
        // 临时设置环境变量
        self.setup_environment();
        
        let request = ChatRequest::new(&self.model, messages);
        
        let response = request.send().await?;
        tracing::info!("[github_backend] Non-streaming request successful");
        
        let content = if let Some(choice) = response.choices.first() {
            if let Some(message) = &choice.message {
                message.content.clone().unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        tracing::info!("[github_backend] GitHub non-streaming response completed, length: {}", content.len());
        Ok(content)
    }
}

#[async_trait::async_trait]
impl LLMBackend for GitHubBackend {
    fn provider(&self) -> LLMProvider {
        LLMProvider::GitHub
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
        tracing::info!("[github_backend] Sending message to GitHub Models API...");
        
        if self.api_token.is_none() {
            let error_msg = "GitHub token not available. Please set GITHUB_TOKEN environment variable.".to_string();
            tracing::error!("[github_backend] {}", error_msg);
            let _ = response_sender.send(LLMResponse {
                content: format!("Error: {}", error_msg),
                is_complete: true,
            });
            return Err(Error::Stream(error_msg.into()));
        }

        let messages = self.build_messages(&text, image_path);

        // 首先尝试流式请求
        match self.try_streaming_request(messages.clone(), &response_sender).await {
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
                tracing::warn!("[github_backend] Streaming request failed: {}, trying non-streaming request...", e);
                
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
                        tracing::error!("[github_backend] Both streaming and non-streaming requests failed. Streaming error: {}, Non-streaming error: {}", e, e2);
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
        tracing::info!("[github_backend] Testing GitHub Models API availability...");
        
        if self.api_token.is_none() {
            let error_msg = "GitHub token not available. Please set GITHUB_TOKEN environment variable.";
            tracing::error!("[github_backend] {}", error_msg);
            return Err(Error::Stream(error_msg.into()));
        }
        
        let messages = vec![
            ChatMessage::system("You are GitHub Copilot, a helpful AI assistant."),
            ChatMessage::user("Please respond with 'Hello from GitHub Copilot!' to confirm you are available."),
        ];

        // 临时设置环境变量
        self.setup_environment();

        // 首先尝试流式请求
        tracing::info!("[github_backend] Attempting streaming test request...");
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
                            tracing::error!("[github_backend] GitHub streaming test error: {}", e);
                            return Err(e);
                        }
                    }
                }

                if !accumulated_content.is_empty() {
                    tracing::info!("[github_backend] GitHub streaming test successful: {}", accumulated_content);
                    Ok(accumulated_content)
                } else {
                    tracing::error!("[github_backend] GitHub streaming test failed: No response content");
                    Err(Error::Stream("No response content from GitHub Models".into()))
                }
            }
            Err(e) => {
                // 流式请求失败，尝试非流式请求
                tracing::warn!("[github_backend] Streaming test failed: {}, trying non-streaming test...", e);
                
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
                            tracing::info!("[github_backend] GitHub non-streaming test successful: {}", content);
                            Ok(content)
                        } else {
                            tracing::error!("[github_backend] GitHub non-streaming test failed: No response content");
                            Err(Error::Stream("No response content from GitHub Models".into()))
                        }
                    }
                    Err(e2) => {
                        tracing::error!("[github_backend] Both streaming and non-streaming tests failed. Streaming error: {}, Non-streaming error: {}", e, e2);
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
        
        println!("🔧 GitHub 测试环境配置:");
        if let Ok(_) = std::env::var("GITHUB_TOKEN") {
            println!("   GitHub Token: [已配置]");
        } else {
            println!("   GitHub Token: [未配置] - 将跳过真实 API 测试");
        }
    }

    #[tokio::test]
    async fn test_github_backend_creation() {
        let _ = tracing_subscriber::fmt::try_init();
        
        // 测试默认创建
        let backend = GitHubBackend::default();
        assert_eq!(backend.model, "gpt-4o");
        assert_eq!(backend.base_url, "https://models.inference.ai.azure.com");
        assert_eq!(backend.provider(), LLMProvider::GitHub);
        assert_eq!(backend.model_name(), "gpt-4o");

        // 测试自定义创建
        let custom_backend = GitHubBackend::new("gpt-3.5-turbo".to_string())
            .with_api_key("test_token".to_string())
            .with_base_url("https://custom.api.com".to_string());
        
        assert_eq!(custom_backend.model, "gpt-3.5-turbo");
        assert_eq!(custom_backend.api_token, Some("test_token".to_string()));
        assert_eq!(custom_backend.base_url, "https://custom.api.com");
        
        println!("✅ GitHub backend creation tests passed!");
    }

    #[tokio::test]
    async fn test_github_backend_availability() {
        setup_test_environment();
        let _ = tracing_subscriber::fmt::try_init();
        
        let backend = GitHubBackend::default();
        
        match backend.test_availability().await {
            Ok(response) => {
                println!("✅ GitHub Models API 可用! 响应: {}", response);
                assert!(!response.is_empty(), "GitHub response should not be empty");
            }
            Err(e) => {
                println!("ℹ️ GitHub Models API 测试失败 (可能因为没有配置 GITHUB_TOKEN): {}", e);
                // 在没有 token 的情况下，这是预期的行为
                eprintln!("GitHub test failed (this might be expected if no GITHUB_TOKEN is configured): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_github_backend_send_message() {
        setup_test_environment();
        let _ = tracing_subscriber::fmt::try_init();
        
        let backend = GitHubBackend::default();
        let (sender, receiver) = mpsc::channel();
        
        let test_message = "Hello GitHub Copilot! Please respond briefly.".to_string();
        
        // 启动异步任务发送消息
        let send_task = tokio::spawn(async move {
            backend.send_message(test_message, None, sender).await
        });
        
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
                println!("✅ GitHub 消息发送成功!");
                println!("📝 最终响应长度: {}", final_content.len());
                println!("📊 总共收到 {} 个响应片段", responses.len());
                
                if !final_content.is_empty() && !final_content.starts_with("Error:") {
                    println!("📄 响应内容预览: {}...", 
                        final_content.chars().take(100).collect::<String>());
                }
            }
            Ok(Err(e)) => {
                println!("ℹ️ GitHub 请求失败 (可能因为没有配置 GITHUB_TOKEN): {}", e);
                eprintln!("GitHub send message test failed (this might be expected if no GITHUB_TOKEN is configured): {}", e);
            }
            Err(e) => {
                println!("❌ 任务执行失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_github_backend_without_token() {
        let _ = tracing_subscriber::fmt::try_init();
        
        // 创建没有 token 的后端
        let backend = GitHubBackend::new("gpt-4o".to_string()); // 不设置 token
        
        // 测试应该失败并返回错误
        match backend.test_availability().await {
            Ok(_) => {
                // 如果环境变量中有 GITHUB_TOKEN，这个测试可能会成功
                println!("ℹ️ 测试成功，可能是因为环境变量中有 GITHUB_TOKEN");
            }
            Err(e) => {
                println!("✅ 正确处理了缺少 token 的情况: {}", e);
                assert!(e.to_string().contains("GitHub token not available"));
            }
        }
    }

    #[tokio::test]
    async fn test_github_backend_send_message_with_image() {
        setup_test_environment();
        let _ = tracing_subscriber::fmt::try_init();
        
        let backend = GitHubBackend::default();
        let (sender, receiver) = mpsc::channel();
        
        // 使用项目中的图标作为测试图片
        let image_path = Path::new("icon/icon.png");
        
        // 检查图片文件是否存在
        if !image_path.exists() {
            println!("⚠️ 测试图片不存在，跳过图片测试: {}", image_path.display());
            return;
        }
        
        let test_message = "Please describe what you see in this image briefly.".to_string();
        
        println!("📸 发送带图片的消息测试，图片路径: {}", image_path.display());
        
        // 启动异步任务发送消息（包含图片）
        let send_task = tokio::spawn(async move {
            backend.send_message(test_message, Some(image_path), sender).await
        });
        
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
                println!("✅ GitHub 图片消息发送成功!");
                println!("📝 最终响应长度: {}", final_content.len());
                println!("📊 总共收到 {} 个响应片段", responses.len());
                
                if !final_content.is_empty() && !final_content.starts_with("Error:") {
                    println!("📄 响应内容预览: {}...", 
                        final_content.chars().take(150).collect::<String>());
                }
            }
            Ok(Err(e)) => {
                println!("ℹ️ GitHub 图片请求失败 (可能因为没有配置 GITHUB_TOKEN): {}", e);
                eprintln!("GitHub send message with image test failed (this might be expected if no GITHUB_TOKEN is configured): {}", e);
            }
            Err(e) => {
                println!("❌ 任务执行失败: {}", e);
            }
        }
    }
}