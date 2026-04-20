use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedConfig {
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WechatConfig {
    pub app_id: String,
    pub app_secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

pub struct CryptoManager {
    config_dir: PathBuf,
    key_path: PathBuf,
    config_path: PathBuf,
    token_path: PathBuf,
}

impl CryptoManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wechat-comments-cli");
        fs::create_dir_all(&config_dir).ok();

        Self {
            config_dir: config_dir.clone(),
            key_path: config_dir.join("key.bin"),
            config_path: config_dir.join("credentials.enc"),
            token_path: config_dir.join("token.json"),
        }
    }

    fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    fn save_key(&self, key: &[u8; 32]) -> Result<(), String> {
        fs::write(&self.key_path, key).map_err(|e| e.to_string())
    }

    fn load_key(&self) -> Result<[u8; 32], String> {
        let bytes = fs::read(&self.key_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "未找到密钥文件，请先运行 'config' 命令进行配置".to_string()
            } else {
                e.to_string()
            }
        })?;

        if bytes.len() != 32 {
            return Err("密钥文件格式错误".to_string());
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    }

    fn key_exists(&self) -> bool {
        self.key_path.exists()
    }

    pub fn save_credentials(&self, app_id: &str, app_secret: &str) -> Result<(), String> {
        let key = if self.key_exists() {
            self.load_key()?
        } else {
            let new_key = Self::generate_key();
            self.save_key(&new_key)?;
            new_key
        };

        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let credentials = WechatConfig {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            access_token: None,
            expires_at: None,
        };

        let plaintext = serde_json::to_string(&credentials).map_err(|e| e.to_string())?;
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| e.to_string())?;

        let encrypted = EncryptedConfig {
            nonce: BASE64.encode(&nonce_bytes),
            ciphertext: BASE64.encode(&ciphertext),
        };

        let json = serde_json::to_string_pretty(&encrypted).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn load_credentials(&self) -> Result<WechatConfig, String> {
        let key = self.load_key()?;

        let content = fs::read_to_string(&self.config_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "未找到配置文件，请先运行 'config' 命令进行配置".to_string()
            } else {
                e.to_string()
            }
        })?;

        let encrypted: EncryptedConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let nonce_bytes = BASE64.decode(&encrypted.nonce).map_err(|e| e.to_string())?;
        let ciphertext = BASE64.decode(&encrypted.ciphertext).map_err(|e| e.to_string())?;

        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "解密失败，配置文件可能已损坏".to_string())?;

        let plaintext_str = String::from_utf8(plaintext).map_err(|e| e.to_string())?;
        let mut config: WechatConfig = serde_json::from_str(&plaintext_str).map_err(|e| e.to_string())?;

        if let Some(token_data) = self.load_token() {
            if token_data.access_token.is_some() && token_data.expires_at.is_some() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if let Some(expires_at) = token_data.expires_at {
                    if now < expires_at - 300 {
                        config.access_token = token_data.access_token;
                        config.expires_at = token_data.expires_at;
                    }
                }
            }
        }

        Ok(config)
    }

    pub fn save_token(&self, access_token: &str, expires_in: u64) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let token_data = WechatConfig {
            app_id: String::new(),
            app_secret: String::new(),
            access_token: Some(access_token.to_string()),
            expires_at: Some(now + expires_in),
        };

        let json = serde_json::to_string_pretty(&token_data).map_err(|e| e.to_string())?;
        fs::write(&self.token_path, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn load_token(&self) -> Option<WechatConfig> {
        if !self.token_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&self.token_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn credentials_exist(&self) -> bool {
        self.config_path.exists() && self.key_path.exists()
    }

    pub fn delete_credentials(&self) -> Result<(), String> {
        if self.key_path.exists() {
            fs::remove_file(&self.key_path).map_err(|e| e.to_string())?;
        }
        if self.config_path.exists() {
            fs::remove_file(&self.config_path).map_err(|e| e.to_string())?;
        }
        if self.token_path.exists() {
            fs::remove_file(&self.token_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn config_dir_exists(&self) -> bool {
        self.config_dir.exists()
    }

    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
}
