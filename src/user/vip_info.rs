use crate::bapi;
use crate::error::BError;
use crate::error::BResult;
use crate::wbi_client::do_request;
use crate::wbi_client::WbiClient;
use crate::ApiGet;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use super::USER_APIS;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VipInfo {
    pub mid: i64,
    pub vip_type: i64,
    pub vip_status: i64,
    pub vip_due_date: i64,
    pub vip_pay_type: i64,
    pub theme_type: i64,
}

#[async_trait]
impl ApiGet for VipInfo {
    type Item = VipInfo;

    async fn get(client: &WbiClient) -> BResult<Self::Item> {
        let req = client.get(bapi!(USER_APIS, "vip_info"));
        let resp = do_request(req).await?;
        let resp = resp.data.ok_or(BError::from_json_err(
            "Invalid json field, data cannot be empty",
        ))?;
        Ok(resp)
    }
}

#[cfg(test)]
mod test {
    use super::VipInfo;
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
        let _info = VipInfo::get(&client).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_get_my_info_panic() {
        let client = WbiClient::builder().build().await.unwrap();
        let _info = VipInfo::get(&client).await.unwrap();
    }
}
