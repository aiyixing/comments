use crate::crypto::{CryptoManager, WechatConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

pub struct WechatApiClient {
    client: Client,
    config: WechatConfig,
    crypto_manager: CryptoManager,
}

impl WechatApiClient {
    pub async fn new() -> Result<Self, String> {
        let crypto_manager = CryptoManager::new();
        
        if !crypto_manager.credentials_exist() {
            return Err("未找到配置，请先运行 'wechat-comments config' 进行配置".to_string());
        }

        let config = crypto_manager.load_credentials()?;
        
        Ok(Self {
            client: Client::new(),
            config,
            crypto_manager,
        })
    }

    pub async fn from_config(config: WechatConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            crypto_manager: CryptoManager::new(),
        }
    }

    pub async fn get_access_token(&mut self) -> Result<String, String> {
        if let Some(token) = &self.config.access_token {
            if let Some(expires_at) = self.config.expires_at {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if now < expires_at - 300 {
                    return Ok(token.clone());
                }
            }
        }

        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.config.app_id, self.config.app_secret
        );

        let response: AccessTokenResponse = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(errcode) = response.errcode {
            return Err(format!("获取token失败: {} - {}", errcode, response.errmsg.unwrap_or_default()));
        }

        let access_token = response.access_token.ok_or("返回数据中无access_token".to_string())?;
        let expires_in = response.expires_in.unwrap_or(7200);

        self.config.access_token = Some(access_token.clone());
        self.config.expires_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + expires_in
        );

        self.crypto_manager.save_token(&access_token, expires_in)?;

        Ok(access_token)
    }

    async fn post<T: Serialize>(&mut self, path: &str, body: &T) -> Result<serde_json::Value, String> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://api.weixin.qq.com{}?access_token={}",
            path, access_token
        );

        let response = self.client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let response_text = response.text().await.map_err(|e| e.to_string())?;
        
        let response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("解析响应失败: {}", e))?;

        if let Some(errcode) = response.get("errcode").and_then(|v| v.as_i64()) {
            if errcode != 0 {
                let errmsg = response.get("errmsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                return Err(format!("API错误: {} - {}", errcode, errmsg));
            }
        }

        Ok(response)
    }

    pub async fn open_comment(&mut self, msg_data_id: u64, index: u32) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/open",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index
            })
        ).await
    }

    pub async fn close_comment(&mut self, msg_data_id: u64, index: u32) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/close",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index
            })
        ).await
    }

    pub async fn list_comments(
        &mut self,
        msg_data_id: u64,
        index: u32,
        begin: u32,
        count: u32,
        comment_type: u32,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/list",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index,
                "begin": begin,
                "count": count,
                "type": comment_type
            })
        ).await
    }

    pub async fn mark_elect_comment(
        &mut self,
        msg_data_id: u64,
        index: u32,
        comment_id: u64,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/markelect",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index,
                "comment_id": comment_id
            })
        ).await
    }

    pub async fn unmark_elect_comment(
        &mut self,
        msg_data_id: u64,
        index: u32,
        comment_id: u64,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/unmarkelect",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index,
                "comment_id": comment_id
            })
        ).await
    }

    pub async fn delete_comment(
        &mut self,
        msg_data_id: u64,
        index: u32,
        comment_id: u64,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/delete",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index,
                "comment_id": comment_id
            })
        ).await
    }

    pub async fn reply_comment(
        &mut self,
        msg_data_id: u64,
        index: u32,
        comment_id: u64,
        content: &str,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/reply/add",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index,
                "comment_id": comment_id,
                "content": content
            })
        ).await
    }

    pub async fn delete_reply(
        &mut self,
        msg_data_id: u64,
        index: u32,
        comment_id: u64,
        reply_id: u64,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/cgi-bin/comment/reply/delete",
            &serde_json::json!({
                "msg_data_id": msg_data_id,
                "index": index,
                "comment_id": comment_id,
                "reply_id": reply_id
            })
        ).await
    }
}
