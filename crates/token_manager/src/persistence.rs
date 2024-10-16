use dashmap::DashMap;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::mem::size_of;
use utils::current_timestamp_s;

use crate::token::{Header, Token, TokenEntry};
use crate::TokenManager;

const PATH: &str = "tokens.bin";

pub struct TokenManagerPersistence;

impl TokenManagerPersistence {
    #[inline]
    #[tracing::instrument(skip_all)]
    pub fn save(token_manager: &TokenManager) -> std::io::Result<()> {
        let mut buffer = Vec::with_capacity(Self::calculate_total_size(token_manager));

        // Write header
        let header = Header {
            tokens_count: token_manager.tokens.len() as u64,
            user_tokens_count: token_manager.user_tokens.len() as u64,
        };
        buffer.extend_from_slice(header.as_bytes());

        // Write tokens
        for entry in token_manager.tokens.iter() {
            let (token, token_entry) = entry.pair();
            buffer.extend_from_slice(token.as_bytes());
            buffer.extend_from_slice(token_entry.as_bytes());
        }

        // Write user_tokens
        for entry in token_manager.user_tokens.iter() {
            let (user_id, tokens) = entry.pair();
            buffer.extend_from_slice(&user_id.to_le_bytes());
            buffer.extend_from_slice(&(tokens.len() as u32).to_le_bytes());
            for token in tokens {
                buffer.extend_from_slice(token.as_bytes());
            }
        }

        // Write the entire buffer to file in one go
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(PATH)?;

        file.write_all(&buffer)?;

        Ok(())
    }

    #[inline]
    #[tracing::instrument(skip_all)]
    pub fn load() -> std::io::Result<TokenManager> {
        let mut buffer: Vec<u8>;

        match File::open(PATH) {
            Ok(mut file) => {
                // Get the file size
                let file_size = file.metadata()?.len() as usize;

                // Read the entire file into a single buffer
                buffer = vec![0u8; file_size];
                file.read_exact(&mut buffer)?;
            }

            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Create new file with header initialized to zeros
                let mut file = File::create(PATH)?;
                let header = Header {
                    tokens_count: 0,
                    user_tokens_count: 0,
                };
                let header_bytes = header.as_bytes();
                file.write_all(header_bytes)?;

                // Return an empty TokenManager since there's nothing else to load
                return Ok(TokenManager {
                    tokens: DashMap::new(),
                    user_tokens: DashMap::new(),
                });
            }
            Err(e) => return Err(e),
        };

        let mut offset = 0;

        // Read header
        let header = Header::from_bytes(&buffer[offset..offset + size_of::<Header>()]);
        offset += size_of::<Header>();

        let tokens = DashMap::with_capacity(header.tokens_count as usize);
        let user_tokens = DashMap::with_capacity(header.user_tokens_count as usize);

        let now = current_timestamp_s();

        // Read tokens
        for _ in 0..header.tokens_count {
            let token = *Token::from_bytes(&buffer[offset..offset + size_of::<Token>()]);
            offset += size_of::<Token>();

            let token_entry =
                *TokenEntry::from_bytes(&buffer[offset..offset + size_of::<TokenEntry>()]);
            offset += size_of::<TokenEntry>();

            if token_entry.expiry > now {
                tokens.insert(token, token_entry);
            }
        }

        // Read user_tokens
        for _ in 0..header.user_tokens_count {
            let user_id = i32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
            offset += 4;

            let tokens_count = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
            offset += 4;

            let mut user_token_list = VecDeque::with_capacity(tokens_count as usize);
            for _ in 0..tokens_count {
                let token = *Token::from_bytes(&buffer[offset..offset + size_of::<Token>()]);
                offset += size_of::<Token>();
                if tokens.contains_key(&token) {
                    user_token_list.push_back(token);
                }
            }

            if !user_token_list.is_empty() {
                user_tokens.insert(user_id, user_token_list);
            }
        }

        Ok(TokenManager {
            tokens,
            user_tokens,
        })
    }

    // Calculate the total size needed for the file
    #[inline]
    #[tracing::instrument(skip_all)]
    fn calculate_total_size(token_manager: &TokenManager) -> usize {
        size_of::<Header>()
            + token_manager.tokens.len() * (size_of::<Token>() + size_of::<TokenEntry>())
            + token_manager
                .user_tokens
                .iter()
                .map(|entry| {
                    size_of::<i32>() + size_of::<u32>() + entry.value().len() * size_of::<Token>()
                })
                .sum::<usize>()
    }
}
