use std::slice;

use base64_simd::{Out, URL_SAFE_NO_PAD};
use error::{AppResult, TokenError};
use ring::rand::{SecureRandom, SystemRandom};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token([u8; 16]);

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum TokenIntent {
    Auth,
    RefreshAuth,
    EmailVerify,
    PasswordReset,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TokenEntry {
    pub(crate) id: i32,
    pub(crate) expiry: u32,
    pub(crate) intent: TokenIntent,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Header {
    pub(crate) tokens_count: u64,
    pub(crate) user_tokens_count: u64,
}

impl TokenIntent {
    pub(crate) const fn lifespan(&self) -> u32 {
        match self {
            TokenIntent::Auth => 24 * 60 * 60,
            TokenIntent::RefreshAuth => 7 * 24 * 60 * 60,
            TokenIntent::EmailVerify => 20 * 60,
            TokenIntent::PasswordReset => 30 * 60,
        }
    }
}

impl Token {
    #[inline]
    pub(crate) fn new() -> Self {
        let rng = SystemRandom::new();
        let mut token = [0u8; 16];

        rng.fill(&mut token)
            .expect("Failed generating random bytes");

        Self(token)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        assert_eq!(bytes.len(), size_of::<Self>());
        assert_eq!(
            bytes.as_ptr() as usize % align_of::<Self>(),
            0,
            "Misaligned pointer"
        );
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }

    #[inline]
    pub fn as_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode_to_string(self.0)
    }

    #[inline]
    pub fn from_base64(str: &str) -> AppResult<Token> {
        let mut token = [0u8; 16];

        match URL_SAFE_NO_PAD.decode(str.as_bytes(), Out::from_slice(&mut token)) {
            Ok(_) => Ok(Self(token)),
            _ => Err(TokenError::InvalidToken)?,
        }
    }
}

impl TokenEntry {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        assert_eq!(bytes.len(), size_of::<Self>());
        assert_eq!(
            bytes.as_ptr() as usize % align_of::<Self>(),
            0,
            "Misaligned pointer"
        );
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }
}

impl Header {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        assert_eq!(bytes.len(), size_of::<Self>());
        assert_eq!(
            bytes.as_ptr() as usize % align_of::<Self>(),
            0,
            "Misaligned pointer"
        );
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }
}
