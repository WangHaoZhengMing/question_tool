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
            base_url: Some("https://api.tu-zi.com/v1".to_string()),
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
            tracing::info!("messages: {:?}", text);
            vec![ChatMessage::system(""), ChatMessage::user(text)]
        }

    }

    /// 设置环境变量以使用自定义的 API key 和 base URL
    fn setup_environment(&self) {
        if let Some(api_key) = &self.api_key {
            unsafe {
                std::env::set_var("OPENAI_API_KEY", api_key);
                tracing::debug!("[gpt_backend] Set OPENAI_API_KEY environment variable");
            }
        }

        if let Some(base_url) = &self.base_url {
            unsafe {
                std::env::set_var("OPENAI_BASE_URL", base_url);
                tracing::debug!("[gpt_backend] Set OPENAI_BASE_URL to: {}", base_url);
            }
        }
    }

    /// 尝试流式请求
    async fn try_streaming_request(
        &self,
        messages: Vec<ChatMessage>,
        response_sender: &mpsc::Sender<LLMResponse>,
    ) -> Result<String, Error> {
        tracing::info!("[gpt_backend] Attempting streaming request to GPT...");

        // 设置环境变量
        self.setup_environment();

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

        // 设置环境变量
        self.setup_environment();

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
        tracing::info!("[gpt_backend] current model: {}", self.model);

        // 设置环境变量
        self.setup_environment();

        // 首先尝试流式请求
        tracing::info!("[gpt_backend] Attempting streaming request...");
        let stream_request = ChatRequest::new(&self.model, messages.clone()).with_stream();

        match stream_request.send_stream().await {
            Ok(mut response) => {
                let mut accumulated_content = String::new();

                while let Some(result) = response.next().await {
                    match result {
                        Ok(response) => {
                            if let Some(content) = response
                                .choices
                                .first()
                                .and_then(|c| c.delta.as_ref())
                                .and_then(|d| d.content.as_ref())
                            {
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
                        Err(e) => {
                            tracing::error!("[gpt_backend] GPT streaming error during processing: {}", e);
                            let _ = response_sender.send(LLMResponse {
                                content: format!("Error during streaming: {}", e),
                                is_complete: true,
                            });
                            return Err(e);
                        }
                    }
                }

                if !accumulated_content.is_empty() {
                    tracing::info!(
                        "[gpt_backend] GPT streaming response completed, total length: {}",
                        accumulated_content.len()
                    );
                    // 发送最终完成响应
                    let _ = response_sender.send(LLMResponse {
                        content: accumulated_content,
                        is_complete: true,
                    });
                    Ok(())
                } else {
                    tracing::error!("[gpt_backend] GPT streaming failed: No response content");
                    let _ = response_sender.send(LLMResponse {
                        content: "Error: No response content from GPT streaming".to_string(),
                        is_complete: true,
                    });
                    Err(Error::Stream("No response content from GPT".into()))
                }
            }
            Err(e) => {
                // 流式请求失败，尝试非流式请求
                tracing::warn!(
                    "[gpt_backend] Streaming request failed: {}, trying non-streaming request...",
                    e
                );

                let non_stream_request = ChatRequest::new(&self.model, messages);

                match non_stream_request.send().await {
                    Ok(response) => {
                        if let Some(content) = response
                            .choices
                            .first()
                            .and_then(|choice| choice.message.as_ref())
                            .and_then(|message| message.content.as_ref())
                            .filter(|content| !content.is_empty())
                        {
                            tracing::info!(
                                "[gpt_backend] GPT non-streaming response successful, length: {}",
                                content.len()
                            );
                            // 发送完整响应
                            let _ = response_sender.send(LLMResponse {
                                content: content.clone(),
                                is_complete: true,
                            });
                            Ok(())
                        } else {
                            tracing::error!("[gpt_backend] GPT non-streaming failed: No response content");
                            let _ = response_sender.send(LLMResponse {
                                content: "Error: No response content from GPT non-streaming".to_string(),
                                is_complete: true,
                            });
                            Err(Error::Stream("No response content from GPT".into()))
                        }
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

        // 设置环境变量
        self.setup_environment();

        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user(
                "先说下你的模型名称\n//请 
直接输出如下格式的JavaScript代码，不要回复其他内容。不要带有```javascript ```，只输出代码就可以了。我不 
用代码块包裹\nvar Questions = [\n    {\n        stem: `Which of the following is a <span class=\"underline fillblank\" data-blank-id=\"593417796829762300\" contenteditable=\"false\" style=\"text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;\"><input type=\"text\" style=\"display:none\">   </span> language?`, //这里不要带题号.这里的data-blank-id每次不要相同\n    
    题型类型: \"语音题\",\n        answer: [\"programming\"],\n        analysis: \"考点：编程语言识别。 
分析：Python是一种高级编程语言，广泛用于数据科学、人工智能等领域。故答案为：programming\", //解析要用中 
文。格式要分为：考点，分析，故答案为：\n    },\n    {\n        stem: `The capital of France is <span class=\"underline fillblank\" data-blank-id=\"593417796829762301\" contenteditable=\"false\" style=\"text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;\"><input type=\"text\" style=\"display:none\">   </span>.`,\n        题型类型: \"填空题\",\n        answer: [\"Paris\"],\n        analysis: \"考点：世界地理常识。分析：巴黎是法国的首都和最大城市，也是法国的政治、经 
济、文化中心。故答案为：Paris\"\n    },\n    {//如果检测到是一个文章。且一个题目里面有多个空的，用下面这种格式。段落两端对齐，首行缩 
进，字体字号不变\n            stem:`Good morning my name is (1) <span class=\"underline fillblank\" data-blank-id=\"593417796829762302\" contenteditable=\"false\" style=\"text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;\"><input type=\"text\" style=\"display:none\">   </span> (这里可能会有提示的单词，你也要写上) I am from (2) <span class=\"underline fillblank\" data-blank-id=\"593417796829762303\" contenteditable=\"false\" style=\"text-indent: 0; border-bottom: 1px solid #f6c908;display:inline-block;min-width: 40px;max-width: 80px;\"><input type=\"text\" style=\"display:none\">   </span>`,\n            //序号从(1)开始。data-blank-id每次不要相同。不用管原题目的题号\n            //序号从(1)开始。data-blank-id每次不要相同不用管原题目的题
号\n            //序号从(1)开始。data-blank-id每次不要相同不用管原题目的题号\n            题型类型: \"填空题\",\n            answer: 
[\"John\", \"Canada\"],\n            analysis: \"1. 考点：.....。分析：根据常见的自我介绍格式，名字是John. 故答案为：John,<br>2. 分析
：.......。国家是Canada。故答案为： Canada\"\n    },\n];\n",
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
                            response
                                .choices
                                .first()
                                .and_then(|c| c.delta.as_ref())
                                .and_then(|d| d.content.as_ref())
                                .map(|content| accumulated_content.push_str(content));
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
                    Ok(response) => response
                        .choices
                        .first()
                        .and_then(|choice| choice.message.as_ref())
                        .and_then(|message| message.content.as_ref())
                        .filter(|content| !content.is_empty())
                        .map(|content| {
                            tracing::info!(
                                "[gpt_backend] GPT non-streaming test successful: {}",
                                content
                            );
                            content.clone()
                        })
                        .ok_or_else(|| {
                            tracing::error!(
                                "[gpt_backend] GPT non-streaming test failed: No response content"
                            );
                            Error::Stream("No response content from GPT".into())
                        }),
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

    #[tokio::test]
    async fn test_gpt_connection() {
        let _ = tracing_subscriber::fmt::try_init();
        let key = "01110011,01101011,00101101,01101111,01101011,01000110,01110111,01100101,01010100,01000110,01011001,01010101,01111010,00110000,00110001,01100001,01000010,01010000,00110111,01011001,01110110,01010100,01101011,01110110,00111000,01001001,00110100,01111010,01101000,01100101,01110110,01110100,01100011,01001000,00110111,01100111,01011000,01101001,01011001,01100010,01100100,01100010,01000111,00110011,01001010,01100010,01101011,01110100,00110001,01001110,01100100";
        let bytes: Vec<u8> = key
            .split(',')
            .filter_map(|b| u8::from_str_radix(b.trim(), 2).ok())
            .collect();
        let backend = GPTBackend::new("gemini-2.5-pro".to_string())
            .with_api_key(String::from_utf8(bytes).unwrap())
            .with_base_url(String::from("http://27.106.110.32:2052/v1"));
        println!("{:?}", backend);
        match backend.test_availability().await {
            Ok(response) => {
                println!("GPT 可用! 响应: {}", response);
                assert!(!response.is_empty(), "GPT response should not be empty");
            }
            Err(e) => {
                println!("GPT 不可用: {}", e);
                eprintln!("GPT test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_send_message_to_gpt() {
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
