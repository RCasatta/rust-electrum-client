
#[derive(Debug, Clone)]
pub struct Config {
    pub socks5: Option<Socks5Config>,
    //pub timeout: u32,
    pub retry: u8,
    //pub validate_domain: bool,
}

#[derive(Debug, Clone)]
pub struct Socks5Config {
    pub addr: String,
    pub credentials: Option<Socks5Credential>,
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

#[derive(Debug, Clone)]
pub struct Socks5Credential {
    pub username: String,
    pub password: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            socks5: None,
            //timeout: 10,
            retry: 1,
            //validate_domain: true,
        }
    }
}
