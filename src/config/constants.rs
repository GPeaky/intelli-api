use std::time::Duration;

// Google auth
pub const GOOGLE_REDIRECT: &str = "https://intellitelemetry.live/auth/google/callback";
pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_USER_INFO: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

// Tokens
pub const GENERIC_TOKEN_EXPIRATION: usize = 15 * 60;
pub const REFRESH_TOKEN_EXPIRATION: usize = 15 * 60;

// Redis
pub const REDIS_USER_PREFIX: &str = "user";
pub const REDIS_CACHE_EXPIRATION: usize = 60 * 60 * 24;
pub const REDIS_CHAMPIONSHIP_PREFIX: &str = "championship";
pub const REDIS_F123_PREFIX: &str = "f123:championships";
pub const REDIS_F123_PERSISTENCE: usize = 15 * 60;

// F123 Service
// Socket
pub const BUFFER_SIZE: usize = 1460;
pub const SOCKET_HOST: &str = "0.0.0.0";
pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(10 * 60);
pub const BATCHING_INTERVAL: Duration = Duration::from_millis(700);

// Session
pub const HISTORY_INTERVAL: Duration = Duration::from_secs(1);
pub const SESSION_INTERVAL: Duration = Duration::from_secs(10);
pub const MOTION_INTERVAL: Duration = Duration::from_millis(700);
