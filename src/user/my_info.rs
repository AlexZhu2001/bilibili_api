use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use crate::bapi;
use crate::error::BError;
use crate::error::BResult;
use crate::wbi_client::do_request;
use crate::ApiGet;

use super::USER_APIS;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MyInfo {
    pub mid: i64,
    pub uname: String,
    pub userid: String,
    pub sign: String,
    pub birthday: String,
    pub sex: String,
    pub nick_free: bool,
    pub rank: String,
}

#[async_trait]
impl ApiGet for MyInfo {
    type Item = MyInfo;

    async fn get(client: &crate::wbi_client::WbiClient) -> BResult<Self::Item> {
        let req = client.get(bapi!(USER_APIS, "my_info"));
        let resp = do_request(req).await?;
        let resp = resp.data.ok_or(BError::from_json_err(
            "Invalid json field, data cannot be empty",
        ))?;
        Ok(resp)
    }
}

#[cfg(test)]
mod test {
    use super::MyInfo;
    use crate::{login::Credential, wbi_client::WbiClient, ApiGet};
    use base64::Engine;
    use std::io::BufReader;

    #[tokio::test]
    async fn test_get_my_info() {
        let cred = std::env::var("CRED_TEST").unwrap();
        let cred = base64::engine::general_purpose::STANDARD
            .decode(&cred)
            .unwrap();
        let rdr = BufReader::new(&cred[..]);
        let mut cred = Credential::load_json(rdr).unwrap();
        let client = WbiClient::builder()
            .with_credential(&mut cred)
            .await
            .unwrap()
            .build()
            .await
            .unwrap();
        let _info = MyInfo::get(&client).await.unwrap();
    }
}
