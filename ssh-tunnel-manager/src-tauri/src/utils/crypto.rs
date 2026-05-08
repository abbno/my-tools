// 加密工具
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

lazy_static::lazy_static! {
    static ref ENCRYPTION_KEY: [u8; KEY_SIZE] = {
        // 在实际应用中，应该从安全的地方获取密钥
        // 这里使用固定的密钥仅作为示例
        *b"ssh-proxy-encryption-key-32byte!"
    };
}

/// 加密数据
pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(&*ENCRYPTION_KEY)
        .map_err(|e| format!("创建加密器失败: {}", e))?;

    let nonce_bytes: [u8; NONCE_SIZE] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("加密失败: {}", e))?;

    // 将 nonce 和 ciphertext 合并并编码为 base64
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);
    Ok(BASE64.encode(&combined))
}

/// 解密数据
pub fn decrypt(encrypted: &str) -> Result<String, String> {
    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    if combined.len() < NONCE_SIZE {
        return Err("无效的加密数据".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&*ENCRYPTION_KEY)
        .map_err(|e| format!("创建解密器失败: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("解密失败: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 解码失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "my_secret_password";
        let encrypted = encrypt(original).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }
}
