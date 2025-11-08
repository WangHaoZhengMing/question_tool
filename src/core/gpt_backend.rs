use std::path::Path;
use super::llm_backend::{LLMBackend, LLMProvider};
use crate::core::utility::img_to_base64_withpath;
use async_llm::Error;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequestArgs,
    },
};

/// GPT 后端实现
#[derive(Clone, Debug)]
pub struct Openai {
    pub model_name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for Openai {
    fn default() -> Self {
        Self {
            model_name: "gpt-4o".to_string(),
            api_key: None,
            base_url: Some("https://api.tu-zi.com/v1".to_string()),
        }
    }
}

impl Openai {
    #[allow(dead_code)]
    pub fn new(model: String) -> Self {
        Self {
            model_name: model,
            api_key: None,
            base_url: None,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn get_base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }
    pub fn get_model_name(&self) -> &str {
        &self.model_name
    }

    /// 构建消息列表
    // fn build_messages(&self, text: &str, image_path: Option<&Path>) -> Vec<ChatMessage> {
    //     if let Some(path) = image_path {
    //         // 如果有图片，转换为 base64
    //         tracing::info!(
    //             "[gpt_backend] Converting image to base64: {}",
    //             path.display()
    //         );
    //         match self.image_to_base64(path) {
    //             Ok(base64) => {
    //                 tracing::info!("[gpt_backend] Image converted to base64 successfully");
    //                 let data_url = format!("data:image/png;base64,{}", base64);
    //                 vec![
    //                     ChatMessage::system(""),
    //                     ChatMessage::user_image_with_text(text, data_url.as_str()),
    //                 ]
    //             }
    //             Err(e) => {
    //                 tracing::error!("[gpt_backend] Failed to convert image to base64: {}", e);
    //                 vec![
    //                     ChatMessage::system(
    //                         "You are a helpful assistant for analyzing questions and images.",
    //                     ),
    //                     ChatMessage::user(text),
    //                 ]
    //             }
    //         }
    //     } else {
    //         // 只有文本
    //         tracing::info!("[gpt_backend] Text-only request");
    //         tracing::info!("messages: {:?}", text);
    //         vec![ChatMessage::system(""), ChatMessage::user(text)]
    //     }
    // }

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
}

#[async_trait::async_trait]
impl LLMBackend for Openai {
    fn provider(&self) -> LLMProvider {
        LLMProvider::GPT
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn send_message(
        &self,
        user_text: String,
        image_path: Option<&Path>,
    ) -> Result<String, Error> {
        // 改为返回 String
        // 设置环境变量（如果需要）
        // self.setup_environment();
        let config = OpenAIConfig::new()
            .with_api_base(self.get_base_url().unwrap_or_default())
            .with_api_key(self.get_api_key().unwrap_or_default());
        let client = Client::with_config(config);
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.get_model_name())
            .max_tokens(512u32)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant.")
                    .build()
                    .map_err(|e| Error::Stream(format!("Failed to build system message: {}", e).into()))?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(
                        if let Some(img_path) = image_path {
                            // If we have an image, create an array with both text and image
                            match img_to_base64_withpath(img_path) {
                                Ok(base64_img) => {
                                    let data_url = format!("data:image/png;base64,{}", base64_img);
                                    ChatCompletionRequestUserMessageContent::Array(vec![
                                        ChatCompletionRequestUserMessageContentPart::Text(
                                            async_openai::types::ChatCompletionRequestMessageContentPartText { text: user_text.clone() },
                                        ),
                                        ChatCompletionRequestUserMessageContentPart::ImageUrl(
                                            ChatCompletionRequestMessageContentPartImage { image_url: data_url.into() }
                                        ),
                                    ])
                                }
                                Err(_) => {
                                    // If image conversion fails, fall back to text only
                                    ChatCompletionRequestUserMessageContent::Text(user_text)
                                }
                            }
                        } else {
                            // Text only
                            ChatCompletionRequestUserMessageContent::Text(user_text)
                        }
                    )
                    .build()
                    .map_err(|e| Error::Stream(format!("Failed to build user message: {}", e).into()))?
                    .into(),
            ])
            .build()
            .map_err(|e| Error::Stream(format!("Failed to build request: {}", e).into()))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::Stream(format!("API request failed: {}", e).into()))?;

        // 从响应中提取文本内容
        let reply = response
            .choices
            .first() // 获取第一个选择
            .and_then(|choice| choice.message.content.as_ref()) // 获取消息内容
            .map(|s| s.to_string()) // 转换为 String
            .ok_or_else(|| Error::Stream("No response content received".into()))?; // 如果没有内容则返回错误

        tracing::info!("[gpt_backend] Received response: {}", reply);

        Ok(reply)
    }

    async fn test_availability(&self) -> Result<String, Error> {
        // 现在 send_message 会返回响应字符串
        let reply = self
            .send_message("hello, check if you work.".to_string(), None)
            .await?;
        Ok(reply)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::utility::decode_binary_to_str;

    use super::*;

    #[tokio::test]
    async fn test_gpt_connection() {
        let _ = tracing_subscriber::fmt::try_init();
        let api_key: &str = "01110011,01101011,00101101,01110101,01000010,01101000,01000101,01101010,01101101,01001110,01010010,00110100,01101110,01001110,01101011,01110110,01110110,01100001,01011010,01110011,01000111,01101100,01100110,01000100,01011000,01101000,01100100,01010110,01110011,01000101,01000110,01001000,01100001,01100011,01110001,01000100,00111000,00110111,01100110,01100011,01100110,01101101,01010110,01110101,01100011,01011001,01001000,01000101,01111010,01011001,01110110";
        let backend = Openai::new("gemini-2.5-pro".to_string())
            .with_api_key(decode_binary_to_str(api_key))
            .with_base_url(String::from("https://api.tu-zi.com/v1"));
        println!("{:?}", backend);
        match backend.test_availability().await {
            Ok(msg) => {
                println!("Connection successful: {}", msg);
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}
