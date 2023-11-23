//! This crate provide some bilibili api
//!
//! * `wbi_client`: Common client for request, store cookie and wbi sign
//!
//! * `login`: Bilibili login api
//!

use serde::{Deserialize, Serialize};
pub mod error;
pub mod login;
pub mod user;
pub mod wbi_client;
use std::collections::HashMap;

pub(crate) type ApiMap = HashMap<&'static str, &'static str>;

#[doc(hidden)]
#[macro_export]
macro_rules! bapi_def {
    ( $x:expr ) => {{
        const API: &'static str = include_str!($x);
        let v: super::ApiMap = serde_json::from_str(API).unwrap();
        v
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! bapi {
    ( $apis:ident, $name:literal ) => {
        $apis[$name]
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct BCommonJson<T>
where
    T: Serialize,
{
    code: i64,
    message: String,
    data: Option<T>,
}

#[cfg(test)]
mod test {
    use super::BCommonJson;

    #[test]
    fn test_json_no_data() {
        let json_str = r#"
            {
                "code": 10086,
                "message": "114514_1919810"
            }
        "#;
        let result: BCommonJson<()> = serde_json::from_str(json_str).unwrap();
        assert_eq!(result.code, 10086);
        assert_eq!(result.message, "114514_1919810");
        assert_eq!(result.data, None);
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    struct SimpleData {
        foo: String,
        baz: f64,
    }

    #[test]
    fn test_json_with_data() {
        let json_str = r#"
            {
                "code": 0,
                "message": "114514_1919810",
                "data": {
                    "foo": "bar",
                    "baz": 114514.1919810
                }
            }
        "#;
        let result: BCommonJson<SimpleData> = serde_json::from_str(json_str).unwrap();
        assert_eq!(result.code, 0);
        assert_eq!(result.message, "114514_1919810");
        let data = result.data.unwrap();
        assert_eq!(data.foo, "bar");
        assert_eq!(data.baz, 114514.1919810);
    }
}
