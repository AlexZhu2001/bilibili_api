use std::sync::Arc;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod sign;

lazy_static! {
    static ref SESSION: Arc<reqwest::Client> = {
        let client = reqwest::Client::new();
        Arc::new(client)
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct BCommonJson<T>
where
    T: Serialize,
{
    code: i64,
    message: String,
    data: T,
}
