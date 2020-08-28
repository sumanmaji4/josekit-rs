use openssl::symm::{self, Cipher};
use anyhow::bail;

use crate::jose::JoseError;
use crate::jwe::JweContentEncryption;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AesGcmJweEncryption {
    /// AES GCM using 128-bit key
    A128Gcm,
    /// AES GCM using 192-bit key
    A192Gcm,
    /// AES GCM using 256-bit key
    A256Gcm,
}

impl AesGcmJweEncryption {
    fn cipher(&self) -> Cipher {
        match self {
            Self::A128Gcm => Cipher::aes_128_gcm(),
            Self::A192Gcm => Cipher::aes_192_gcm(),
            Self::A256Gcm => Cipher::aes_256_gcm(),
        }
    }
}

impl JweContentEncryption for AesGcmJweEncryption {
    fn name(&self) -> &str {
        match self {
            Self::A128Gcm => "A128GCM",
            Self::A192Gcm => "A192GCM",
            Self::A256Gcm => "A256GCM",
        }
    }

    fn key_len(&self) -> usize {
        match self {
            Self::A128Gcm => 16,
            Self::A192Gcm => 24,
            Self::A256Gcm => 32,
        }
    }

    fn iv_len(&self) -> usize {
        12
    }

    fn encrypt(&self, key: &[u8], iv: Option<&[u8]>, message: &[u8], aad: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>), JoseError> {
        (|| -> anyhow::Result<(Vec<u8>, Option<Vec<u8>>)> {
            let expected_len = self.key_len();
            if key.len() != expected_len {
                bail!(
                    "The length of content encryption key must be {}: {}",
                    expected_len,
                    key.len()
                );
            }

            let cipher = self.cipher();
            let mut tag = [0; 16];
            let encrypted_message = symm::encrypt_aead(cipher, key, iv, aad, message, &mut tag)?;
            Ok((encrypted_message, Some(tag.to_vec())))
        })()
        .map_err(|err| JoseError::InvalidKeyFormat(err))
    }

    fn decrypt(&self, key: &[u8], iv: Option<&[u8]>, encrypted_message: &[u8], aad: &[u8], tag: Option<&[u8]>) -> Result<Vec<u8>, JoseError> {
        (|| -> anyhow::Result<Vec<u8>> {
            let expected_len = self.key_len();
            if key.len() != expected_len {
                bail!(
                    "The length of content encryption key must be {}: {}",
                    expected_len,
                    key.len()
                );
            }

            let tag = match tag {
                Some(val) => val,
                None => bail!("A tag value is required."),
            };

            let cipher = self.cipher();
            let message = symm::decrypt_aead(cipher, key, iv, aad, encrypted_message, tag)?;
            Ok(message)
        })()
        .map_err(|err| JoseError::InvalidJweFormat(err))
    }

    fn box_clone(&self) -> Box<dyn JweContentEncryption> {
        Box::new(self.clone())
    }
}
