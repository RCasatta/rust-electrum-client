use crate::Error;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    /// Proxy socks5 configuration, default None
    socks5: Option<Socks5Config>,
    /// timeout in seconds, default None (depends on TcpStream default)
    timeout: Option<Duration>,
    /// number of retry if any error, default 1
    retry: u8,
    /// when ssl, validate the domain, default true
    validate_domain: bool,
}

#[derive(Debug, Clone)]
pub struct Socks5Config {
    pub addr: String,
    pub credentials: Option<Socks5Credential>,
}

#[derive(Debug, Clone)]
pub struct Socks5Credential {
    pub username: String,
    pub password: String,
}

pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            config: Config::default(),
        }
    }

    pub fn socks5(mut self, socks5_config: Option<Socks5Config>) -> Result<Self, Error> {
        if socks5_config.is_some() && self.config.timeout.is_some() {
            return Err(Error::BothSocksAndTimeout);
        }
        self.config.socks5 = socks5_config;
        Ok(self)
    }

    pub fn timeout(mut self, timeout: u8) -> Result<Self, Error> {
        if self.config.socks5.is_some() {
            return Err(Error::BothSocksAndTimeout);
        }
        self.config.timeout = Some(Duration::from_secs(timeout as u64));
        Ok(self)
    }

    pub fn retry(mut self, retry: u8) -> Self {
        self.config.retry = retry;
        self
    }

    pub fn validate_domain(mut self, validate_domain: bool) -> Self {
        self.config.validate_domain = validate_domain;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Socks5Config {
    pub fn new(addr: impl ToString) -> Self {
        let addr = addr.to_string().replacen("socks5://", "", 1);
        Socks5Config {
            addr,
            credentials: None,
        }
    }

    pub fn with_credentials(addr: String, username: String, password: String) -> Self {
        let mut config = Socks5Config::new(addr);
        config.credentials = Some(Socks5Credential { username, password });
        config
    }
}

impl Config {
    pub fn socks5(&self) -> &Option<Socks5Config> {
        &self.socks5
    }
    pub fn retry(&self) -> u8 {
        self.retry
    }
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
    pub fn validate_domain(&self) -> bool {
        self.validate_domain
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            socks5: None,
            timeout: None,
            retry: 1,
            validate_domain: true,
        }
    }
}
