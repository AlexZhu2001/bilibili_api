//! This module provides the WbiClient type for various requests
//!
//! WbiClient provides `get`, `get_with_data` and `get_with_wbi` functions for GET requests
//!
//! * `get` for no query data
//! * `get_with_data` for normal queries
//! * `get_with_wbi` for queries sign by wbi key

mod sign;

use self::sign::WbiSign;
use crate::{
    error::{BError, BResult},
    login::Credential,
    BCommonJson,
};
use reqwest::{Client, ClientBuilder, IntoUrl, RequestBuilder};
use reqwest_cookie_store::{CookieStore, CookieStoreRwLock};
use serde::{de::DeserializeOwned, Serialize};
use std::{io::BufReader, sync::Arc};

/// Wbi client for api request
pub struct WbiClient {
    client: Client,
    cookies: Arc<CookieStoreRwLock>,
    wbi_key: WbiSign,
}

impl WbiClient {
    /// Creates a `WbiClientBuilder` to configure a `WbiClient`
    ///
    /// # Examples
    /// ```
    /// # use bilibili_api::wbi_client::*;
    /// #[tokio::main]
    /// # async fn main() {
    /// let c = WbiClient::builder().build().await.unwrap();
    /// # }
    /// ```
    pub fn builder() -> WbiClientBuilder {
        WbiClientBuilder::new()
    }

    /// Create a GET request builder to a URL with no query to transfer.
    ///
    /// # Examples
    /// ```
    /// # use bilibili_api::wbi_client::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let c = WbiClient::builder().build().await.unwrap();
    /// c.get("https://bilibili.com");
    /// # }
    /// ```
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        let req = self.client.get(url);
        req
    }

    /// Create a GET request builder to a URL with queries to transfer.
    ///
    /// # Examples
    /// ```
    /// # use bilibili_api::wbi_client::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let c = WbiClient::builder().build().await.unwrap();
    /// c.get_with_data("https://bilibili.com", &[("foo", "bar")]);
    /// # }
    /// ```
    pub fn get_with_data<U: IntoUrl, T: Serialize + ?Sized>(
        &self,
        url: U,
        query: &T,
    ) -> RequestBuilder {
        let req = self.client.get(url).query(query);
        req
    }

    /// Create a GET request builder to a URL with queries signed with wbi.
    ///
    /// # Examples
    /// ```
    /// # use bilibili_api::wbi_client::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let c = WbiClient::builder().build().await.unwrap();
    /// c.get_with_wbi("https://bilibili.com", &[("foo", "bar")]);
    /// # }
    /// ```
    pub fn get_with_wbi<U: IntoUrl, T: Serialize + ?Sized>(
        &self,
        url: U,
        query: &T,
    ) -> BResult<RequestBuilder> {
        let req = self.client.get(url);
        let req = self.wbi_key.sign_data(req, query)?;
        return Ok(req);
    }

    pub(crate) fn get_cookies(&self) -> BResult<String> {
        let mut cookies = Vec::new();
        self.cookies
            .read()
            .map_err(|e| BError::from_internal_err(&e))?
            .save_json(&mut cookies)
            .map_err(|e| BError::from_internal_err(&e))?;
        let cookies = String::from_utf8(cookies).map_err(|e| BError::from_internal_err(&e))?;
        Ok(cookies)
    }
}

/// A `WbiClientBuilder` can be used to create a `WbiClient` with custom configuration.
pub struct WbiClientBuilder {
    cb: ClientBuilder,
    cookies: Option<Arc<CookieStoreRwLock>>,
    wbi_key: Option<WbiSign>,
}

impl WbiClientBuilder {
    fn new() -> Self {
        Self {
            cb: Client::builder(),
            cookies: None,
            wbi_key: None,
        }
    }

    /// Set credential to WbiClient, Credential may be refreshed after calling this function,
    /// you should save the credential after calling this method
    #[must_use]
    pub async fn with_credential(self, c: &mut Credential) -> BResult<Self> {
        let mut tmp = self;
        let cookie_jar = {
            let json = BufReader::new(c.cookies.as_bytes());
            let c = CookieStore::load_json(json).map_err(|e| BError::from_internal_err(&e))?;
            let c = CookieStoreRwLock::new(c);
            let c = Arc::new(c);
            c
        };
        let client = Client::builder()
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()
            .map_err(|e| BError::from_internal_err(&e))?;

        c.check_and_refresh(&client, Arc::clone(&cookie_jar))
            .await?;

        tmp.cookies = Some(cookie_jar);
        Ok(tmp)
    }

    /// Build Client
    ///     
    /// # Examples
    /// ```
    /// # use bilibili_api::wbi_client::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let c = WbiClient::builder().build().await.unwrap();
    /// # }
    /// ```
    #[must_use]
    pub async fn build(self) -> BResult<WbiClient> {
        let cookie_provider = match self.cookies {
            Some(c) => c,
            None => {
                let c = CookieStore::default();
                let c = CookieStoreRwLock::new(c);
                let c = Arc::new(c);
                c
            }
        };
        let client = self
            .cb
            .cookie_provider(Arc::clone(&cookie_provider))
            .build()
            .map_err(|e| BError::from_internal_err(&e))?;
        let wbi_key = match self.wbi_key {
            Some(k) => k,
            None => WbiSign::from_server(&client).await?,
        };
        Ok(WbiClient {
            client: client,
            cookies: cookie_provider,
            wbi_key: wbi_key,
        })
    }
}

pub(crate) async fn do_request<T: Serialize + DeserializeOwned>(
    req: RequestBuilder,
) -> BResult<BCommonJson<T>> {
    let resp = req.send().await.map_err(|e| BError::from_net_err(&e))?;
    let obj = resp.json().await.map_err(|e| BError::from_json_err(&e))?;
    Ok(obj)
}
