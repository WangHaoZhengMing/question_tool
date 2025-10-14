use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::core::llm_backend::LLMManager;

/// LLM 设置配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub github_token: Option<String>,
    pub enable_streaming: bool,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "GPT".to_string(),
            model: "gpt-4o".to_string(),
            api_key: None,
            base_url: None,
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            enable_streaming: true,
        }
    }
}

/// LLM 设置管理器
pub struct LLMSettingsManager {
    config: LLMConfig,
    manager: LLMManager,
    config_path: PathBuf,
}

impl LLMSettingsManager {
    /// 创建新的设置管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_config_from_file(&config_path)?;
        let manager = Self::create_llm_manager(&config);

        Ok(Self {
            config,
            manager,
            config_path,
        })
    }

    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir())
            .ok_or("无法找到配置目录")?;
        
        config_dir.push("question_tool");
        
        // 确保目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        config_dir.push("llm_config.json");
        Ok(config_dir)
    }

    /// 从文件加载配置
    fn load_config_from_file(path: &PathBuf) -> Result<LLMConfig, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: LLMConfig = serde_json::from_str(&content)?;
            tracing::info!("[llm_settings] 已加载配置: {:?}", config.provider);
            Ok(config)
        } else {
            tracing::info!("[llm_settings] 配置文件不存在，使用默认配置");
            Ok(LLMConfig::default())
        }
    }

    /// 创建 LLM 管理器
    fn create_llm_manager(config: &LLMConfig) -> LLMManager {
        LLMManager::from_config(config)
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &LLMConfig {
        &self.config
    }

    /// 更新提供商
    pub fn set_provider(&mut self, provider: String) -> Result<(), Box<dyn std::error::Error>> {
        self.config.provider = provider;
        self.update_manager()?;
        Ok(())
    }

    /// 更新模型
    pub fn set_model(&mut self, model: String) -> Result<(), Box<dyn std::error::Error>> {
        self.config.model = model;
        self.update_manager()?;
        Ok(())
    }

    /// 更新 API Key
    pub fn set_api_key(&mut self, api_key: String) -> Result<(), Box<dyn std::error::Error>> {
        self.config.api_key = if api_key.is_empty() { None } else { Some(api_key) };
        self.update_manager()?;
        Ok(())
    }

    /// 更新 Base URL
    pub fn set_base_url(&mut self, base_url: String) -> Result<(), Box<dyn std::error::Error>> {
        self.config.base_url = if base_url.is_empty() { None } else { Some(base_url) };
        self.update_manager()?;
        Ok(())
    }

    /// 更新 GitHub Token
    pub fn set_github_token(&mut self, token: String) -> Result<(), Box<dyn std::error::Error>> {
        self.config.github_token = if token.is_empty() { None } else { Some(token) };
        self.update_manager()?;
        Ok(())
    }

    /// 更新流式设置
    pub fn set_streaming(&mut self, enable: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.config.enable_streaming = enable;
        Ok(())
    }

    /// 更新管理器配置
    fn update_manager(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.manager = LLMManager::from_config(&self.config);
        Ok(())
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<String, String> {
        tracing::info!("[llm_settings] 开始测试连接...");
        
        match self.manager.test_current_backend().await {
            Ok(response) => {
                let success_msg = format!("✅ 连接成功!\n提供商: {}\n模型: {}\n响应: {}", 
                    self.config.provider, 
                    self.config.model, 
                    response.chars().take(100).collect::<String>()
                );
                tracing::info!("[llm_settings] 连接测试成功");
                Ok(success_msg)
            }
            Err(e) => {
                let error_msg = format!("❌ 连接失败!\n错误: {}", e);
                tracing::error!("[llm_settings] 连接测试失败: {}", e);
                Err(error_msg)
            }
        }
    }

    /// 保存配置到文件
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        tracing::info!("[llm_settings] 配置已保存到: {:?}", self.config_path);
        Ok(())
    }

    /// 重新加载配置
    pub fn reload_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = Self::load_config_from_file(&self.config_path)?;
        self.update_manager()?;
        tracing::info!("[llm_settings] 配置已重新加载");
        Ok(())
    }

    /// 获取支持的模型列表
    #[allow(dead_code)]
    pub fn get_supported_models(&self, provider: &str) -> Vec<String> {
        match provider {
            "GPT" => vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            "GitHub" => vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            _ => vec!["gpt-4o".to_string()],
        }
    }

    /// 获取配置摘要
    pub fn get_config_summary(&self) -> String {
        format!(
            "提供商: {} | 模型: {} | 流式: {} | API Key: {} | GitHub Token: {}",
            self.config.provider,
            self.config.model,
            if self.config.enable_streaming { "启用" } else { "禁用" },
            if self.config.api_key.is_some() { "已配置" } else { "未配置" },
            if self.config.github_token.is_some() { "已配置" } else { "未配置" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_settings_manager() {
        let _ = tracing_subscriber::fmt::try_init();
        
        // 创建设置管理器
        let mut manager = match LLMSettingsManager::new() {
            Ok(m) => m,
            Err(e) => {
                println!("创建设置管理器失败: {}", e);
                return;
            }
        };

        // 测试配置更新
        manager.set_provider("GitHub".to_string()).unwrap();
        assert_eq!(manager.get_config().provider, "GitHub");

        manager.set_model("gpt-3.5-turbo".to_string()).unwrap();
        assert_eq!(manager.get_config().model, "gpt-3.5-turbo");

        manager.set_streaming(false).unwrap();
        assert_eq!(manager.get_config().enable_streaming, false);

        // 测试保存和重新加载
        if let Err(e) = manager.save_config() {
            println!("保存配置失败: {}", e);
        }

        if let Err(e) = manager.reload_config() {
            println!("重新加载配置失败: {}", e);
        }

        // 打印配置摘要
        println!("配置摘要: {}", manager.get_config_summary());

        // 测试连接（可能会失败，取决于是否配置了 API key）
        match manager.test_connection().await {
            Ok(result) => println!("连接测试成功: {}", result),
            Err(e) => println!("连接测试失败 (这是正常的，如果没有配置 API key): {}", e),
        }

        println!("✅ 设置管理器测试完成");
    }
}