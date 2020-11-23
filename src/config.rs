#[derive(Debug, Clone)]
pub struct Config {
    socks5: Option<Socks5Config>,
    timeout: u32,
    retry: u8,
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

    pub fn socks5(mut self, socks5_config: Socks5Config) -> Self {
        self.config.socks5 = Some(socks5_config);
        self
    }

    pub fn timeout(mut self, timeout: u32) -> Self {
        self.config.timeout = timeout;
        self
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

impl Socks5Config {
    pub fn new(addr: impl ToString) -> Self {
        Socks5Config {
            addr: addr.to_string(),
            credentials: None,
        }
    }

    pub fn with_credentials(addr: String, username: String, password: String) -> Self {
        Socks5Config {
            addr,
            credentials: Some(Socks5Credential { username, password }),
        }
    }
}

impl Config {
    pub fn socks5(&self) -> &Option<Socks5Config> {
        &self.socks5
    }
    pub fn retry(&self) -> u8 {
        self.retry
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            socks5: None,
            timeout: 15,
            retry: 1,
            validate_domain: true,
        }
    }
}
