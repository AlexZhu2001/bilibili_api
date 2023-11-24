use crate::{
    error::{BError, BResult},
    BCommonJson,
};
use chrono::{Days, FixedOffset, NaiveDateTime, NaiveTime, Utc};
use md5::{Digest, Md5};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

#[cfg(not(test))]
fn get_timestamp() -> BResult<u64> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| BError::from_internal_err(&e))?;
    return Ok(ts.as_secs());
}

#[cfg(test)]
fn get_timestamp() -> BResult<u64> {
    return Ok(1684746387u64); // Only for test
}

// Part of Nav api data, only the fields wbi needed
#[derive(Debug, Serialize, Deserialize)]
struct WbiImg {
    img_url: String,
    sub_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PartialNav {
    wbi_img: WbiImg,
}

/// Convert url into key
///
/// url form like `https://i0.hdslb.com/bfs/wbi/<key>.png`
///
/// so split '/' and get last one and split '.' then return first one
fn url_to_key(url: &str) -> Option<&str> {
    let tmp = url.split('/').last()?;
    let tmp = tmp.split('.').nth(0)?;
    Some(tmp)
}

/// Get timestamp of next day 00:00 (UTC +8)
///
/// Get now time in East +8 and add one day, set time to 00:00
///
/// Then get a timestamp
fn get_next_day() -> BResult<u64> {
    const HOUR: i32 = 3600;
    // TZ UTC+8
    let east_8 = FixedOffset::east_opt(8 * HOUR).ok_or(BError::InternalError(String::from(
        "Cannot get timezone East +8.",
    )))?;
    // Now time
    let now = Utc::now();
    // Time in UTC+8
    let china = now.with_timezone(&east_8);
    let date = china.date_naive();
    // Get next day date
    let next_day = match date.checked_add_days(Days::new(1)) {
        Some(d) => d,
        None => {
            return Err(BError::InternalError(String::from(
                "Cannot get next day timestamp.",
            )))
        }
    };
    // Get next day time 00:00
    let day_start = match NaiveTime::from_hms_opt(0, 0, 0) {
        Some(t) => t,
        None => {
            return Err(BError::InternalError(String::from(
                "Cannot get next day timestamp.",
            )))
        }
    };
    // Set to naive datetime
    let next_day = NaiveDateTime::new(next_day, day_start)
        .and_utc()
        .timestamp();
    // Invalid time if negative
    if next_day < 0 {
        Err(BError::InternalError(String::from(
            "Next day timestamp is invalid.",
        )))
    } else {
        Ok(next_day as u64)
    }
}

/// Since March 2023, wbi authentication method was needed for some api of bilibili
///
/// Signature Algorithm:
///
/// 1. Get `img_url` and `sub_url` from bilibili nav api
/// 2. Extract `img_key` and `sub_key` from previous step result
/// 3. Concat `img_key` and `sub_key` and rearrange with constant encrypt table
/// 4. Take first 32 chars and set to `mixin_key`
/// 5. Add timestamp with key `wts` and sort query pairs
/// 6. Encode query pairs with url encode and add `mixin_key` to the end as salt
/// 7. Calc hash with MD5 algorithm and add into query with key `w_rid`
///
/// The 1-4 steps were implemented in `from_server` function
///
/// And other steps were implemented in `sign_data` function
///
/// You can cache this object and reuse it in the same day TZ(UTC+8)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WbiSign {
    mixin_key: String,
    expire_time: u64,
}

impl WbiSign {
    pub(crate) fn new(mixin_key: String, expire_time: u64) -> WbiSign {
        Self {
            mixin_key,
            expire_time,
        }
    }

    /// Get wbi sign from bilibili server
    pub async fn from_server(client: &Client) -> BResult<WbiSign> {
        const MIXIN_KEY_ENC_TAB: [usize; 64] = [
            46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42,
            19, 29, 28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60,
            51, 30, 4, 22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
        ];

        const URL: &str = "https://api.bilibili.com/x/web-interface/nav";
        let req: BCommonJson<PartialNav> = client
            .get(URL)
            .send()
            .await
            .map_err(|e| BError::from_net_err(&e))?
            .json()
            .await
            .map_err(|e| BError::from_json_err(&e))?;
        let data = req.data.ok_or(BError::from_json_err(
            "Invalid json field, data cannot be empty",
        ))?;
        let (img_url, sub_url) = (&data.wbi_img.img_url, &data.wbi_img.sub_url);
        let (img_key, sub_key) = match (url_to_key(img_url), url_to_key(sub_url)) {
            (Some(i), Some(s)) => (i, s),
            _ => return Err(BError::from_json_err("Invalid wbi key format.")),
        };
        let mixin_key = String::from(img_key) + sub_key;
        let mixin_key = {
            let mut v = ['\0'; 64];
            for (c, &idx) in mixin_key.chars().zip(MIXIN_KEY_ENC_TAB.iter()) {
                v[idx] = c;
            }
            String::from_iter(v.iter())
        };
        let expired = get_next_day()?;
        Ok(WbiSign {
            mixin_key: mixin_key,
            expire_time: expired,
        })
    }

    /// Sign request data with wbi key
    ///
    /// `req`: RequestBuilder by reqwest crate
    ///
    /// `data`: Query data
    ///
    /// If wbi key is expired will return error `BError::WbiTokenExpired`
    pub fn sign_data<T>(&self, req: RequestBuilder, data: &T) -> BResult<RequestBuilder>
    where
        T: Serialize + ?Sized,
    {
        // Check if Wbi key is expired
        let now = get_timestamp().map_err(|_| BError::WbiTokenExpired)?;
        if now >= self.expire_time {
            return Err(BError::WbiTokenExpired);
        }
        // Convert data into query pairs
        let query_str =
            serde_urlencoded::to_string(data).map_err(|e| BError::from_internal_err(&e))?;
        let mut v: Vec<(&str, &str)> =
            serde_urlencoded::from_str(&query_str).map_err(|e| BError::from_internal_err(&e))?;
        // Insert wts data
        let ts = now.to_string();
        v.push(("wts", &ts));
        // Sort by key
        v.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        // Url encode queries
        let mut query_str =
            serde_urlencoded::to_string(&v).map_err(|e| BError::from_internal_err(&e))?;
        // Add mixin key as salt
        query_str.push_str(&self.mixin_key);
        // MD5 hash
        let mut md5 = Md5::new();
        md5.update(query_str);
        let w_rid = md5.finalize();
        let w_rid = format!("{:x}", w_rid);
        // Add w_rid query
        v.push(("w_rid", &w_rid));
        // Add queries into request builder
        Ok(req.query(&v))
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_sign() {
        use super::{get_timestamp, WbiSign};
        // Test case
        const MIXIN_KEY: &str = "72136226c6a73669787ee4fd02a74c27";
        const DATA: [(&str, &str); 3] = [("foo", "114"), ("bar", "514"), ("zab", "1919810")];
        const RESULT: &str = "90efcab09403023875b8516f07e9f9de";
        // Test Logic
        let s = WbiSign {
            mixin_key: String::from(MIXIN_KEY),
            expire_time: u64::MAX,
        };
        let client = reqwest::Client::new();
        let rq = client.get("http://useless.net");
        let rq = s.sign_data(rq, &DATA).unwrap();
        let rq = rq.build().unwrap();
        let wts = rq.url().query_pairs().find(|(k, _)| k.eq("wts")).unwrap();
        let real_wts = format!("{}", get_timestamp().unwrap());
        let w_rid = rq.url().query_pairs().find(|(k, _)| k.eq("w_rid")).unwrap();
        let real_w_rid = RESULT;
        assert_eq!(wts.1, real_wts);
        assert_eq!(w_rid.1, real_w_rid);
    }
}
