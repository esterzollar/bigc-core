use super::Interpreter;
use crate::tokens::{Token, TokenType};
use aes::Aes256;
use base64::{engine::general_purpose, Engine as _};
use block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use rand::{thread_rng, Rng};

type Aes256CbcDec = Decryptor<Aes256>;
type Aes256CbcEnc = Encryptor<Aes256>;

impl Interpreter {
    pub fn handle_bit(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "bit"
        if *i >= tokens.len() {
            return;
        }

        match tokens[*i].token_type {
            TokenType::Code => self.handle_bit_code(i, tokens),
            TokenType::Decode => self.handle_bit_decode(i, tokens),
            TokenType::Aes => self.handle_bit_aes(i, tokens),
            TokenType::Demon => self.handle_bit_demon(i, tokens),
            _ => {}
        }
    }

    fn skip_braces_and_get_value(&self, i: &mut usize, tokens: &Vec<Token>) -> String {
        if tokens[*i].token_type == TokenType::LBrace {
            *i += 1;
            let val = self.get_token_value(&tokens[*i]);
            *i += 1; // At RBrace
            val
        } else {
            self.get_token_value(&tokens[*i])
        }
    }

    fn handle_bit_code(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "code"
        let text_raw = self.skip_braces_and_get_value(i, tokens);
        let text = self.interpolate_string(&text_raw);
        *i += 1;
        if *i < tokens.len() && tokens[*i].token_type == TokenType::With {
            *i += 1;
            let key_raw = self.skip_braces_and_get_value(i, tokens);
            let key = self.interpolate_string(&key_raw);
            *i += 1;
            if *i < tokens.len() && tokens[*i].token_type == TokenType::Nebc {
                let result = self.holmes_math_cipher(&text, &key, true);
                self.handle_set_as_multiple(i, tokens, vec![result]);
            }
        }
    }

    fn handle_bit_decode(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "decode"
        let text_raw = self.skip_braces_and_get_value(i, tokens);
        let text = self.interpolate_string(&text_raw);
        *i += 1;
        if *i < tokens.len() && tokens[*i].token_type == TokenType::With {
            *i += 1;
            let key_raw = self.skip_braces_and_get_value(i, tokens);
            let key = self.interpolate_string(&key_raw);
            *i += 1;
            if *i < tokens.len() && tokens[*i].token_type == TokenType::Nebc {
                let result = self.holmes_math_cipher(&text, &key, false);
                self.handle_set_as_multiple(i, tokens, vec![result]);
            }
        }
    }

    fn handle_bit_demon(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "demon"
        let text = self.skip_braces_and_get_value(i, tokens);

        let mut result = String::new();
        let mut rng = thread_rng();
        let chars: Vec<char> =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()"
                .chars()
                .collect();

        for c in text.chars() {
            result.push(c);
            for _ in 0..2 {
                result.push(chars[rng.gen_range(0..chars.len())]);
            }
        }
        self.handle_set_as_multiple(i, tokens, vec![result]);
    }

    fn handle_bit_aes(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "aes"
        let mode = tokens[*i].token_type.clone();
        *i += 1; // Skip encrypt/decrypt
        let data = self.skip_braces_and_get_value(i, tokens);
        *i += 1;

        if *i < tokens.len() && tokens[*i].token_type == TokenType::Key {
            *i += 1;
            let key_str = self.skip_braces_and_get_value(i, tokens);
            *i += 1;
            if *i < tokens.len() && tokens[*i].token_type == TokenType::Iv {
                *i += 1;
                let iv_str = self.skip_braces_and_get_value(i, tokens);

                let result = if mode == TokenType::Decrypt {
                    self.aes_decrypt(&data, &key_str, &iv_str)
                } else {
                    self.aes_encrypt(&data, &key_str, &iv_str)
                };
                self.handle_set_as_multiple(i, tokens, vec![result]);
            }
        }
    }

    fn holmes_math_cipher(&self, text: &str, key: &str, encrypt: bool) -> String {
        if encrypt {
            let mut rng = BigRng::new(key);
            let bytes = text.as_bytes();
            let mut out = Vec::with_capacity(bytes.len());
            for b in bytes {
                out.push(b ^ rng.next_byte());
            }
            hex::encode(out)
        } else if let Ok(bytes) = hex::decode(text) {
            let mut rng = BigRng::new(key);
            let mut out = Vec::with_capacity(bytes.len());
            for b in bytes {
                out.push(b ^ rng.next_byte());
            }
            String::from_utf8(out).unwrap_or_else(|_| String::from("Error: Invalid UTF-8"))
        } else {
            String::from("Error: Invalid Hex")
        }
    }

    fn aes_encrypt(&self, data: &str, key_str: &str, iv_str: &str) -> String {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        let k_bytes = key_str.as_bytes();
        let i_bytes = iv_str.as_bytes();
        for i in 0..32 {
            if i < k_bytes.len() {
                key[i] = k_bytes[i];
            }
        }
        for i in 0..16 {
            if i < i_bytes.len() {
                iv[i] = i_bytes[i];
            }
        }

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let data_bytes = data.as_bytes();
        let mut buf = vec![0u8; data_bytes.len() + 32];
        buf[..data_bytes.len()].copy_from_slice(data_bytes);

        match cipher.encrypt_padded_mut::<Pkcs7>(&mut buf, data_bytes.len()) {
            Ok(ct) => general_purpose::STANDARD.encode(ct),
            Err(_) => String::from("Error: Encrypt Fail"),
        }
    }

    fn aes_decrypt(&self, data_b64: &str, key_str: &str, iv_str: &str) -> String {
        let encrypted_data = match general_purpose::STANDARD.decode(data_b64) {
            Ok(d) => d,
            Err(_) => return String::from("Error: Base64 Fail"),
        };
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        let k_bytes = key_str.as_bytes();
        let i_bytes = iv_str.as_bytes();
        for i in 0..32 {
            if i < k_bytes.len() {
                key[i] = k_bytes[i];
            }
        }
        for i in 0..16 {
            if i < i_bytes.len() {
                iv[i] = i_bytes[i];
            }
        }

        let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
        let mut buf = encrypted_data.clone();
        match cipher.decrypt_padded_mut::<Pkcs7>(&mut buf) {
            Ok(pt) => String::from_utf8_lossy(pt).to_string(),
            Err(_) => String::from("Error: Decrypt Fail"),
        }
    }
}

// Simple deterministic PRNG for NEBC Stream Cipher (PCG-like)
struct BigRng {
    state: u64,
}
impl BigRng {
    fn new(key: &str) -> Self {
        // FNV-1a Hash to seed the state
        let mut h: u64 = 0xcbf29ce484222325;
        for b in key.bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
        Self { state: h }
    }
    fn next_byte(&mut self) -> u8 {
        // LCG Step
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // High XOR shift for better randomness in lower bits
        ((self.state ^ (self.state >> 18)) >> 27) as u8
    }
}
