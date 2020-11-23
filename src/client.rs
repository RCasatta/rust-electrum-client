//! Electrum Client

use std::sync::RwLock;

use log::warn;

use bitcoin::{Script, Txid};

use api::ElectrumApi;
use batch::Batch;
use raw_client::*;

use config::Config;
use types::*;

/// Generalized Electrum client that supports multiple backends. This wraps
/// [`RawClient`](client/struct.RawClient.html) and provides a more user-friendly
/// constructor that can choose the right backend based on the url prefix.
///
/// **This is available only with the `default` features, or if `proxy` and one ssl implementation are enabled**
pub enum ClientType {
    #[doc(hidden)]
    TCP(RawClient<ElectrumPlaintextStream>),
    #[doc(hidden)]
    SSL(RawClient<ElectrumSslStream>),
    #[doc(hidden)]
    Socks5(RawClient<ElectrumProxyStream>),
}

pub struct Client {
    client_type: RwLock<ClientType>,
    config: Config,
    url: String,
}

macro_rules! impl_inner_call {
    ( $self:expr, $name:ident $(, $args:expr)* ) => {
    {
        let mut count = 0;
        let mut errors = Vec::with_capacity(count as usize);
        loop {
            if count == $self.config.retry {
                return Err(Error::AllAttemptsErrored(errors));
            }
            count += 1;
            let read_client = $self.client_type.read().unwrap();
            let res = match &*read_client {
                ClientType::TCP(inner) => inner.$name( $($args, )* ),
                ClientType::SSL(inner) => inner.$name( $($args, )* ),
                ClientType::Socks5(inner) => inner.$name( $($args, )* ),
            };
            drop(read_client);
            match res {
                Ok(val) => return Ok(val),
                Err(err) => {
                    let mut write_client = $self.client_type.write().unwrap();
                    warn!("retry:{} {:?}", count, err);
                    errors.push(err);
                    (*write_client) = ClientType::from_config(&$self.url, &$self.config)?;
                },
            }

            std::thread::sleep(std::time::Duration::from_secs(count as u64));
        }}
    }
}

impl ClientType {
    pub fn from_config(url: &str, config: &Config) -> Result<Self, Error> {
        // let socks5 = socks5.map(|s| s.replacen("socks5://", "", 1)); TODO move in Socks5Config

        if url.starts_with("ssl://") {
            if config.socks5.is_some() {
                return Err(Error::SSLOverSocks5);
            }

            let url = url.replacen("ssl://", "", 1);
            let client = RawClient::new_ssl(url.as_str(), true)?;
            Ok(ClientType::SSL(client))
        } else {
            let url = url.replacen("tcp://", "", 1);

            Ok(match config.socks5.as_ref() {
                None => ClientType::TCP(RawClient::new(url.as_str())?),
                Some(socks5) => ClientType::Socks5(RawClient::new_proxy(url.as_str(), socks5)?),
            })
        }
    }
}

impl Client {
    /// Default constructor supporting multiple backend by providing a prefix
    ///
    /// Supported prefixes are:
    /// - tcp:// for a TCP plaintext client.
    /// - ssl:// for an SSL-encrypted client. The server certificate will be verified.
    ///
    /// If no prefix is specified, then `tcp://` is assumed.
    ///
    /// See [Client::from_config] for more configuration options
    ///
    pub fn new(url: &str) -> Result<Self, Error> {
        Self::from_config(url, Config::default())
    }

    /// Generic constructor that supports multiple backends and allows configuration through
    /// the [Config]
    ///
    /// **NOTE**: SSL-over-socks5 is currently not supported and will generate a runtime error.
    ///
    pub fn from_config(url: &str, config: Config) -> Result<Self, Error> {
        let client_type = RwLock::new(ClientType::from_config(url, &config)?);

        Ok(Client {
            client_type,
            config,
            url: url.to_string(),
        })
    }
}

impl ElectrumApi for Client {
    #[inline]
    fn batch_call(&self, batch: &Batch) -> Result<Vec<serde_json::Value>, Error> {
        impl_inner_call!(self, batch_call, batch)
    }

    #[inline]
    fn block_headers_subscribe_raw(&self) -> Result<RawHeaderNotification, Error> {
        impl_inner_call!(self, block_headers_subscribe_raw)
    }

    #[inline]
    fn block_headers_pop_raw(&self) -> Result<Option<RawHeaderNotification>, Error> {
        impl_inner_call!(self, block_headers_pop_raw)
    }

    #[inline]
    fn block_header_raw(&self, height: usize) -> Result<Vec<u8>, Error> {
        impl_inner_call!(self, block_header_raw, height)
    }

    #[inline]
    fn block_headers(&self, start_height: usize, count: usize) -> Result<GetHeadersRes, Error> {
        impl_inner_call!(self, block_headers, start_height, count)
    }

    #[inline]
    fn estimate_fee(&self, number: usize) -> Result<f64, Error> {
        impl_inner_call!(self, estimate_fee, number)
    }

    #[inline]
    fn relay_fee(&self) -> Result<f64, Error> {
        impl_inner_call!(self, relay_fee)
    }

    #[inline]
    fn script_subscribe(&self, script: &Script) -> Result<Option<ScriptStatus>, Error> {
        impl_inner_call!(self, script_subscribe, script)
    }

    #[inline]
    fn script_unsubscribe(&self, script: &Script) -> Result<bool, Error> {
        impl_inner_call!(self, script_unsubscribe, script)
    }

    #[inline]
    fn script_pop(&self, script: &Script) -> Result<Option<ScriptStatus>, Error> {
        impl_inner_call!(self, script_pop, script)
    }

    #[inline]
    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error> {
        impl_inner_call!(self, script_get_balance, script)
    }

    #[inline]
    fn batch_script_get_balance<'s, I>(&self, scripts: I) -> Result<Vec<GetBalanceRes>, Error>
    where
        I: IntoIterator<Item = &'s Script> + Clone,
    {
        impl_inner_call!(self, batch_script_get_balance, scripts.clone())
    }

    #[inline]
    fn script_get_history(&self, script: &Script) -> Result<Vec<GetHistoryRes>, Error> {
        impl_inner_call!(self, script_get_history, script)
    }

    #[inline]
    fn batch_script_get_history<'s, I>(&self, scripts: I) -> Result<Vec<Vec<GetHistoryRes>>, Error>
    where
        I: IntoIterator<Item = &'s Script> + Clone,
    {
        impl_inner_call!(self, batch_script_get_history, scripts.clone())
    }

    #[inline]
    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error> {
        impl_inner_call!(self, script_list_unspent, script)
    }

    #[inline]
    fn batch_script_list_unspent<'s, I>(
        &self,
        scripts: I,
    ) -> Result<Vec<Vec<ListUnspentRes>>, Error>
    where
        I: IntoIterator<Item = &'s Script> + Clone,
    {
        impl_inner_call!(self, batch_script_list_unspent, scripts.clone())
    }

    #[inline]
    fn transaction_get_raw(&self, txid: &Txid) -> Result<Vec<u8>, Error> {
        impl_inner_call!(self, transaction_get_raw, txid)
    }

    #[inline]
    fn batch_transaction_get_raw<'t, I>(&self, txids: I) -> Result<Vec<Vec<u8>>, Error>
    where
        I: IntoIterator<Item = &'t Txid> + Clone,
    {
        impl_inner_call!(self, batch_transaction_get_raw, txids.clone())
    }

    #[inline]
    fn batch_block_header_raw<'s, I>(&self, heights: I) -> Result<Vec<Vec<u8>>, Error>
    where
        I: IntoIterator<Item = u32> + Clone,
    {
        impl_inner_call!(self, batch_block_header_raw, heights.clone())
    }

    #[inline]
    fn batch_estimate_fee<'s, I>(&self, numbers: I) -> Result<Vec<f64>, Error>
    where
        I: IntoIterator<Item = usize> + Clone,
    {
        impl_inner_call!(self, batch_estimate_fee, numbers.clone())
    }

    #[inline]
    fn transaction_broadcast_raw(&self, raw_tx: &[u8]) -> Result<Txid, Error> {
        impl_inner_call!(self, transaction_broadcast_raw, raw_tx)
    }

    #[inline]
    fn transaction_get_merkle(&self, txid: &Txid, height: usize) -> Result<GetMerkleRes, Error> {
        impl_inner_call!(self, transaction_get_merkle, txid, height)
    }

    #[inline]
    fn server_features(&self) -> Result<ServerFeaturesRes, Error> {
        impl_inner_call!(self, server_features)
    }

    #[inline]
    fn ping(&self) -> Result<(), Error> {
        impl_inner_call!(self, ping)
    }

    #[inline]
    #[cfg(feature = "debug-calls")]
    fn calls_made(&self) -> usize {
        impl_inner_call!(self, calls_made)
    }
}
