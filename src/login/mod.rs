//! This module provides functions and structures about login

use crate::{
    bapi, bapi_def,
    error::{BError, BResult},
    wbi_client::do_request,
    ApiMap, BCommonJson,
};
use lazy_static::lazy_static;
use reqwest::Client;
use reqwest_cookie_store::CookieStoreRwLock;
use rsa::{pkcs8::DecodePublicKey, sha2::Sha256, Oaep, RsaPublicKey};
use select::{document::Document, predicate::Attr};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, Write},
    sync::Arc,
};

// Sub mods
mod qrcode;

lazy_static! {
    static ref LOGIN_APIS: ApiMap = bapi_def!("login.json");
}

/// Structure for persistent storage of cookies and refresh_token
#[derive(Debug, Deserialize, Serialize)]
pub struct Credential {
    pub(crate) cookies: String,
    pub(crate) refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshCheck {
    refresh: bool,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshToken {
    refresh_token: String,
}

/// Base-16 encode lowercase
fn hex_digest(v: &Vec<u8>) -> String {
    const ENC_TAB: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    ];
    let out: String = v
        .iter()
        .flat_map(|x| {
            let high = (x >> 4) as usize;
            let low = (x & 0xF) as usize;
            [ENC_TAB[high], ENC_TAB[low]]
        })
        .collect();
    out
}

/// Generate correspond path
/// # Steps
/// 1. Encrypt `refresh_{timestamp}` with RSA-OAEP(SHA256)
/// 2. Encode result with Base-16 lowercase
fn gen_correspond_path(ts: u64) -> BResult<String> {
    let mut rng = rand::thread_rng();
    let pem = include_str!("correspond_path.pem");
    let public_key =
        RsaPublicKey::from_public_key_pem(pem).map_err(|e| BError::from_internal_err(&e))?;
    let oaep = Oaep::new::<Sha256>();
    let token = format!("refresh_{}", ts);
    let enc_data = public_key
        .encrypt(&mut rng, oaep, token.as_bytes())
        .map_err(|e| BError::from_internal_err(&e))?;
    Ok(hex_digest(&enc_data))
}

/// Get refresh CSRF from server with refresh token
async fn get_refresh_csrf(client: &Client, token: &str) -> BResult<String> {
    let url = bapi!(LOGIN_APIS, "get_refresh_csrf_template");
    let mut url = String::from(url);
    url.push_str(token);
    let req = client.get(url);
    let text = req
        .send()
        .await
        .map_err(|e| BError::from_net_err(&e))?
        .text()
        .await
        .map_err(|e| BError::from_internal_err(&e))?;
    let doc = Document::from(&text[..]);
    let node = doc
        .find(Attr("id", "1-name"))
        .nth(0)
        .ok_or(BError::InternalError(String::from("Cannot get 1-name.")))?;
    Ok(node.text())
}

/// Check if cookie need refresh
async fn check_cookie(client: &Client) -> BResult<RefreshCheck> {
    let req = client.get(bapi!(LOGIN_APIS, "check_refresh"));
    let resp = do_request(req).await?;
    if resp.code != 0 {
        return Err(BError::from_bilibili_err(resp.code));
    }
    let data: RefreshCheck = resp.data.ok_or(BError::from_json_err(
        "Invalid json field, data cannot be empty",
    ))?;
    return Ok(data);
}

/// Do refresh with csrf
async fn refresh_cookie(
    client: &Client,
    csrf: &str,
    refresh_csrf: &str,
    old_token: &str,
) -> BResult<String> {
    let req = client.post(bapi!(LOGIN_APIS, "refresh_cookie"));
    let req = req.form(&[
        ("csrf", csrf),
        ("refresh_csrf", refresh_csrf),
        ("source", "main_web"),
        ("refresh_token", old_token),
    ]);
    let resp: BCommonJson<RefreshToken> = do_request(req).await?;
    if resp.code != 0 {
        return Err(BError::from_bilibili_err(resp.code));
    }
    let new_refresh_token = resp
        .data
        .ok_or(BError::from_json_err(
            "Invalid json field, data cannot be empty",
        ))?
        .refresh_token;
    Ok(new_refresh_token)
}

/// Confirm refresh is complete, invalid old refresh token
async fn confirm_refresh(client: &Client, refresh_csrf: &str, old_token: &str) -> BResult<()> {
    let req = client.post(bapi!(LOGIN_APIS, "confirm_refresh"));
    let req = req.form(&[("csrf", refresh_csrf), ("refresh_token", old_token)]);
    let resp: BCommonJson<()> = req
        .send()
        .await
        .map_err(|e| BError::from_net_err(&e))?
        .json()
        .await
        .map_err(|e| BError::from_json_err(&e))?;
    if resp.code != 0 {
        return Err(BError::from_bilibili_err(resp.code));
    }
    Ok(())
}

/// Get bilibili cookie from cookie jar with given name
fn get_bilibili_cookie(cookie_jar: Arc<CookieStoreRwLock>, name: &str) -> BResult<String> {
    let lock = cookie_jar
        .read()
        .map_err(|e| BError::from_internal_err(&e))?;
    let c = lock
        .get("bilibili.com", "/", name)
        .ok_or(BError::InternalError(String::from(
            "No bili_jct in original cookies, please re-login",
        )))?
        .value();
    Ok(String::from(c))
}

impl Credential {
    /// Save credential in json with writer
    ///
    /// # Examples
    /// ```rust
    /// # use bilibili_api::login::*;
    /// #
    /// # fn main(){
    /// # let data = r#"{"cookies": "test_c", "refresh_token": "test_t"}"#.as_bytes();
    /// # let reader = std::io::BufReader::new(data);
    /// # let c = Credential::load_json(reader).unwrap();
    /// # let output:Vec<u8> = Vec::new();
    /// let mut writer = std::io::BufWriter::new(output);
    /// c.save_json(& mut writer);
    /// # }
    /// ```
    pub fn save_json<W: Write>(&self, w: &mut W) -> BResult<()> {
        serde_json::to_writer(w, self).map_err(|e| BError::from_internal_err(&e))?;
        Ok(())
    }

    /// Load credential in json with reader
    ///
    /// # Examples
    /// ```rust
    /// # use bilibili_api::login::*;
    /// # use reqwest_cookie_store::{CookieStore, CookieStoreRwLock};
    /// #
    /// # fn main(){
    /// # let cookies = CookieStore::default();
    /// # let mut saved = Vec::new();
    /// # cookies.save_json(&mut saved).unwrap();
    /// # let cookies = String::from_utf8(saved).unwrap();
    /// # let data = format!(r#"{{"cookies":"{}", "refresh_token":"123"}}"#, cookies);
    /// let reader = std::io::BufReader::new(data.as_bytes());
    /// let c = Credential::load_json(reader).unwrap();
    /// # }
    /// ```
    pub fn load_json<R: BufRead>(r: R) -> BResult<Self> {
        let c = serde_json::from_reader(r).map_err(|e| BError::from_internal_err(&e))?;
        Ok(c)
    }

    /// Check and refresh credential when needed
    /// # Steps
    /// 1. Check if refresh is required
    /// 2. Using timestamp generate correspond path
    /// 3. Get refresh csrf from server
    /// 4. Get csrf from cookie jar
    /// 5. Get new refresh token from server
    /// 6. Refresh cookie with refresh_token, refresh_csrf and cookie
    /// 7. Confirm refresh with new cookie and old refresh token
    pub(crate) async fn check_and_refresh(
        &mut self,
        client: &Client,
        cookie_jar: Arc<CookieStoreRwLock>,
    ) -> BResult<()> {
        // Bind previous credential
        let prev = self;

        // Check if refresh is required
        let data = check_cookie(client).await?;
        if !data.refresh {
            return Ok(());
        }

        // Generate Correspond Path with RSA-OAEP(SHA-256)
        let cp = gen_correspond_path(data.timestamp)?;

        // Get new csrf from server
        let refresh_csrf = get_refresh_csrf(client, &cp).await?;

        // Get old csrf from cookie jar
        let csrf = get_bilibili_cookie(Arc::clone(&cookie_jar), "bili_jct")?;

        // Get new refresh token and new cookies
        let new_refresh_token =
            refresh_cookie(client, &csrf, &refresh_csrf, &prev.refresh_token).await?;

        // Confirm refresh is complete, old refresh token is going to invalid after this op
        confirm_refresh(client, &refresh_csrf, &prev.refresh_token).await?;

        // Save new cookies and refresh token
        let mut w = Vec::new();
        cookie_jar
            .read()
            .map_err(|e| BError::from_internal_err(&e))?
            .save_json(&mut w)
            .map_err(|e| BError::from_internal_err(&e))?;

        prev.cookies = String::from_utf8(w).map_err(|e| BError::from_internal_err(&e))?;
        prev.refresh_token = new_refresh_token;
        Ok(())
    }
}

// Re-export
pub use self::qrcode::{QRCodeLogin, QRCodeLoginState};
