use std::fmt::Display;
use std::path::{ Path};
use std::sync::mpsc;

use async_llm::Error;

/// LLM 响应结构
#[derive(Clone, Debug)]
pub struct LLMResponse {
    pub content: String,
    pub is_complete: bool,
}

/// LLM 提供商枚举
#[derive(Clone, Debug, PartialEq)]
pub enum LLMProvider {
    GPT,
    GitHub,
}
impl Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProvider::GPT => write!(f, "GPT"),
            LLMProvider::GitHub => write!(f, "GitHub"),
        }
    }
}
/// 通用 LLM 后端 trait
#[async_trait::async_trait]
pub trait LLMBackend: Send + Sync {
    fn provider(&self) -> LLMProvider;

    fn model_name(&self) -> &str;
    
    async fn send_message(
        &self,
        text: String,
        image_path: Option<&Path>,
        response_sender: mpsc::Sender<LLMResponse>,
    ) -> Result<(), Error>;
    
    /// 测试 LLM 是否可用
    async fn test_availability(&self) -> Result<String, Error>;
}

use super::gpt_backend::GPTBackend;
use super::github_backend::GitHubBackend;

/// LLM 管理器，负责管理不同的 LLM 后端
pub struct LLMManager {
    backends: Vec<Box<dyn LLMBackend>>,
    current_backend: Option<usize>,
}

impl LLMManager {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            current_backend: None,
        }
    }

     pub fn from_config(config: &crate::app::llm_settings::LLMConfig) -> Self {
        let mut manager = Self::new();

        // 添加 GPT 后端
        let mut gpt_backend = GPTBackend::default();
        if let Some(api_key) = &config.api_key {
            gpt_backend = gpt_backend.with_api_key(api_key.clone());
        }
        if let Some(base_url) = &config.base_url {
            if !base_url.is_empty() {
                gpt_backend = gpt_backend.with_base_url(base_url.clone());
            }
        }
        gpt_backend.model = config.model.clone();

        let gpt_index = manager.add_backend(Box::new(gpt_backend));

        // 添加 GitHub 后端
        let mut github_backend = GitHubBackend::new(config.model.clone());
        if let Some(token) = &config.github_token {
            github_backend = github_backend.with_api_key(token.clone());
        }

        let github_index = manager.add_backend(Box::new(github_backend));

        // 设置当前后端
        match config.provider.as_str() {
            "GPT" => {
                let _ = manager.set_current_backend(gpt_index);
            }
            "GitHub" => {
                let _ = manager.set_current_backend(github_index);
            }
            _ => {
                let _ = manager.set_current_backend(gpt_index);
            }
        }

        manager
    }
    /// 添加后端
    pub fn add_backend(&mut self, backend: Box<dyn LLMBackend>) -> usize {
        let index = self.backends.len();
        self.backends.push(backend);
        
        // 如果是第一个后端，设置为当前后端
        if self.current_backend.is_none() {
            self.current_backend = Some(index);
        }
        
        index
    }

    /// 设置当前后端
    pub fn set_current_backend(&mut self, index: usize) -> Result<(), String> {
        if index < self.backends.len() {
            self.current_backend = Some(index);
            Ok(())
        } else {
            Err(format!("Backend index {} out of range", index))
        }
    }

    /// 获取当前后端
    pub fn current_backend(&self) -> Option<&dyn LLMBackend> {
        self.current_backend
            .and_then(|index| self.backends.get(index))
            .map(|backend| backend.as_ref())
    }

    /// 列出所有后端
    #[allow(dead_code)]
    pub fn list_backends(&self) -> Vec<(usize, LLMProvider, &str)> {
        self.backends
            .iter()
            .enumerate()
            .map(|(index, backend)| (index, backend.provider(), backend.model_name()))
            .collect()
    }

    /// 发送消息到当前后端
    pub async fn send_message(
        &self,
        text: String,
        image_path: Option<&Path>,
        response_sender: mpsc::Sender<LLMResponse>,
    ) -> Result<(), Error> {
        if let Some(backend) = self.current_backend() {
            tracing::info!("Sending message to LLM backend: {}", backend.provider());
            backend.send_message(text, image_path, response_sender).await
        } else {
            Err(Error::Stream("No backend available".into()))
        }
    }

    /// 测试当前后端可用性
    pub async fn test_current_backend(&self) -> Result<String, Error> {
        if let Some(backend) = self.current_backend() {
            backend.test_availability().await
        } else {
            Err(Error::Stream("No backend available".into()))
        }
    }
}

impl Default for LLMManager {
    fn default() -> Self {
        // 使用默认配置创建管理器
        use crate::app::llm_settings::LLMConfig;
        Self::from_config(&LLMConfig::default())
    }
}


/// 为了向后兼容，保留原有的函数接口
pub async fn send_message_to_llm(
    text: String,
    image_path: Option<&Path>,
    response_sender: mpsc::Sender<LLMResponse>,
) -> Result<(), Error> {
    let manager = LLMManager::default();
    manager.send_message(text, image_path, response_sender).await
}




#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_manager() {
        let _ = tracing_subscriber::fmt::try_init();
        
        let mut manager = LLMManager::new();
        
        // 添加 GPT 后端
        let gpt_backend = Box::new(GPTBackend::default());
        let gpt_index = manager.add_backend(gpt_backend);
        
        // 测试后端列表
        let backends = manager.list_backends();
        assert_eq!(backends.len(), 1);
        assert_eq!(backends[0].1, LLMProvider::GPT);
        
        // 测试设置当前后端
        assert!(manager.set_current_backend(gpt_index).is_ok());
        assert!(manager.set_current_backend(99).is_err());
        
        // 测试获取当前后端
        assert!(manager.current_backend().is_some());
        
        println!("✅ LLM Manager tests passed!");
    }
}

