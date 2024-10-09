use base64_simd::{Out, URL_SAFE_NO_PAD};
use ring::rand::{SecureRandom, SystemRandom};

const TOKEN_SIZE: usize = 16;

pub enum TokenError {
    InvalidLength,
    DecodingError(base64_simd::Error)
}

#[repr(transparent)]
struct Token([u8; TOKEN_SIZE]);

impl Token {
    fn new() -> Self {
        let mut token = [0u8; TOKEN_SIZE];
        SystemRandom::new().fill(&mut token).expect("rng failed");
        Token(token)
    }

    fn to_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode_to_string(self.0)
    }

    fn from_base64(s: &str) -> Result<Self, Error> {
        let mut token = [0u8; TOKEN_SIZE];
        URL_SAFE_NO_PAD.decode(s.as_bytes(), Out::from_slice(&mut token))
    }

}
