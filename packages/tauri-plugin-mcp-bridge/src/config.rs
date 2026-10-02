//! Configuration for the MCP Bridge plugin.
//!
//! This module provides configuration options for customizing the plugin behavior,
//! including the WebSocket server bind address and the authentication token source.

use std::net::IpAddr;

/// Default bind address. The bridge only ever listens on loopback.
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// Default environment variable holding the bearer token clients must present.
pub const DEFAULT_TOKEN_ENV: &str = "MCP_BRIDGE_TOKEN";

/// Minimum accepted token length (32 chars of base64url/hex ≈ 128+ bits of entropy).
const MIN_TOKEN_LENGTH: usize = 32;

/// Maximum accepted token length, to bound header parsing.
const MAX_TOKEN_LENGTH: usize = 512;

/// Configuration for the MCP Bridge plugin.
#[derive(Clone, Debug)]
pub struct Config {
    /// The address to bind the WebSocket server to.
    /// Must be a loopback address; anything else is rejected at startup and
    /// replaced with "127.0.0.1".
    pub bind_address: String,
    /// The base port for the WebSocket server.
    /// Default: 9223. The plugin will scan up to 100 ports from this base.
    pub base_port: u16,
    /// Name of the environment variable that holds the bearer token.
    /// Default: "MCP_BRIDGE_TOKEN". The server refuses to start without a valid token.
    pub auth_token_env: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: DEFAULT_BIND_ADDRESS.to_string(),
            base_port: 9223,
            auth_token_env: DEFAULT_TOKEN_ENV.to_string(),
        }
    }
}

impl Config {
    /// Creates a new configuration with the specified bind address.
    ///
    /// Non-loopback addresses are rejected at startup.
    pub fn new(bind_address: &str) -> Self {
        Self {
            bind_address: bind_address.to_string(),
            ..Self::default()
        }
    }

    /// Creates a configuration that binds to localhost only (same as the default).
    pub fn localhost_only() -> Self {
        Self::default()
    }
}

/// Returns true if `addr` is a loopback IP address (127.0.0.0/8 or ::1).
pub(crate) fn is_loopback_address(addr: &str) -> bool {
    addr.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

/// Reason a token failed validation. Never contains the token itself.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenError {
    Missing,
    TooShort,
    TooLong,
    InvalidCharacters,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "token is not set"),
            Self::TooShort => write!(f, "token must be at least {MIN_TOKEN_LENGTH} characters"),
            Self::TooLong => write!(f, "token must be at most {MAX_TOKEN_LENGTH} characters"),
            Self::InvalidCharacters => write!(f, "token may only contain [A-Za-z0-9_-]"),
        }
    }
}

/// Validates a bearer token's shape.
pub(crate) fn validate_token(token: &str) -> Result<(), TokenError> {
    if token.is_empty() {
        return Err(TokenError::Missing);
    }
    if token.len() < MIN_TOKEN_LENGTH {
        return Err(TokenError::TooShort);
    }
    if token.len() > MAX_TOKEN_LENGTH {
        return Err(TokenError::TooLong);
    }
    if !token
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(TokenError::InvalidCharacters);
    }
    Ok(())
}

/// Reads and validates the bearer token from the named environment variable.
pub(crate) fn load_token(env_name: &str) -> Result<String, TokenError> {
    let token = std::env::var(env_name).map_err(|_| TokenError::Missing)?;
    validate_token(&token)?;
    Ok(token)
}

/// Builder for creating a configured MCP Bridge plugin.
///
/// The bridge always binds to loopback and requires a bearer token read from
/// an environment variable (`MCP_BRIDGE_TOKEN` by default).
///
/// # Examples
///
/// ```rust,ignore
/// use tauri_plugin_mcp_bridge::Builder;
///
/// // Default: 127.0.0.1, token from MCP_BRIDGE_TOKEN
/// let plugin: tauri::plugin::TauriPlugin<tauri::Wry> = Builder::new().build();
///
/// // Custom token variable:
/// let plugin: tauri::plugin::TauriPlugin<tauri::Wry> = Builder::new()
///     .auth_token_env("VISAO_MCP_TOKEN")
///     .build();
/// ```
pub struct Builder {
    config: Config,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Sets the bind address for the WebSocket server.
    ///
    /// Only loopback addresses (e.g. "127.0.0.1", "::1") are accepted; any other
    /// value is rejected at startup and replaced with "127.0.0.1".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tauri_plugin_mcp_bridge::Builder;
    ///
    /// let builder = Builder::new().bind_address("127.0.0.1");
    /// ```
    pub fn bind_address(mut self, addr: &str) -> Self {
        self.config.bind_address = addr.to_string();
        self
    }

    /// Sets the base port for the WebSocket server.
    ///
    /// The plugin will scan up to 100 ports starting from this base port.
    ///
    /// # Arguments
    ///
    /// * `port` - The base port number (e.g., 9223)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tauri_plugin_mcp_bridge::Builder;
    ///
    /// let builder = Builder::new().base_port(9323);
    /// ```
    pub fn base_port(mut self, port: u16) -> Self {
        self.config.base_port = port;
        self
    }

    /// Sets the environment variable the bearer token is read from.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tauri_plugin_mcp_bridge::Builder;
    ///
    /// let builder = Builder::new().auth_token_env("VISAO_MCP_TOKEN");
    /// ```
    pub fn auth_token_env(mut self, env_name: &str) -> Self {
        self.config.auth_token_env = env_name.to_string();
        self
    }

    /// Builds the plugin with the configured options.
    pub fn build<R: tauri::Runtime>(self) -> tauri::plugin::TauriPlugin<R> {
        crate::init_with_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_detection() {
        assert!(is_loopback_address("127.0.0.1"));
        assert!(is_loopback_address("127.1.2.3"));
        assert!(is_loopback_address("::1"));
        assert!(!is_loopback_address("0.0.0.0"));
        assert!(!is_loopback_address("192.168.1.10"));
        assert!(!is_loopback_address("::"));
        assert!(!is_loopback_address("localhost"));
        assert!(!is_loopback_address(""));
    }

    #[test]
    fn test_validate_token() {
        let good = "a".repeat(MIN_TOKEN_LENGTH);
        assert_eq!(validate_token(&good), Ok(()));
        assert_eq!(validate_token("Abc_-019".repeat(8).as_str()), Ok(()));
        assert_eq!(validate_token(""), Err(TokenError::Missing));
        assert_eq!(validate_token("short"), Err(TokenError::TooShort));
        assert_eq!(
            validate_token(&"a".repeat(MAX_TOKEN_LENGTH + 1)),
            Err(TokenError::TooLong)
        );
        let bad = format!("{} b", "a".repeat(MIN_TOKEN_LENGTH));
        assert_eq!(validate_token(&bad), Err(TokenError::InvalidCharacters));
    }

    #[test]
    fn test_default_is_loopback() {
        assert!(is_loopback_address(&Config::default().bind_address));
    }
}
