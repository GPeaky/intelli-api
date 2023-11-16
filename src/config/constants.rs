use std::time::Duration;

// F123 Service
// Socket
pub const SOCKET_HOST: &str = "0.0.0.0";
pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(10 * 60);

pub const BUFFER_SIZE: usize = 1460;

// Session
pub const HISTORY_INTERVAL: Duration = Duration::from_secs(1);
pub const SESSION_INTERVAL: Duration = Duration::from_secs(10);
pub const MOTION_INTERVAL: Duration = Duration::from_millis(700);
