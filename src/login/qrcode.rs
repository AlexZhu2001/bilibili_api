//! This sub-mod provides function and types of login with qrcode

use super::{Credential, LOGIN_APIS};
use crate::{
    bapi,
    error::{BError, BResult},
    wbi_client::{do_request, WbiClient},
};
use qrcode::QrCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QRCodeLogin {
    url: String,
    qrcode_key: String,
}

#[derive(Debug)]
pub enum QRCodeLoginState {
    Success(Credential),
    QRCodeExpired,
    WaitConfirm,
    WaitScan,
}

#[derive(Debug, Deserialize, Serialize)]
struct QRCodeLoginPoll {
    code: i64,
    refresh_token: String,
}

impl QRCodeLogin {
    #[must_use]
    pub async fn get_login_info(wbi_client: &WbiClient) -> BResult<Self> {
        let req = wbi_client.get(bapi!(LOGIN_APIS, "get_qrcode"));
        let obj = do_request(req).await?;
        Ok(obj.data.ok_or(BError::from_json_err(
            "Invalid json field, data cannot be empty",
        ))?)
    }

    pub fn get_login_qrcode(&self) -> BResult<QrCode> {
        QrCode::new(&self.url).map_err(|e| BError::from_qrcode_err(&e))
    }

    #[must_use]
    #[cfg(not(tarpaulin_include))]
    pub async fn poll_login_state(&self, wbi_client: &WbiClient) -> BResult<QRCodeLoginState> {
        let data = [("qrcode_key", &self.qrcode_key)];
        let req = wbi_client.get_with_data(bapi!(LOGIN_APIS, "poll_qrcode"), &data);
        let obj = do_request(req).await?;
        let poll: QRCodeLoginPoll = obj.data.ok_or(BError::from_json_err(
            "Invalid json field, data cannot be empty",
        ))?;
        let state = match poll.code {
            0 => {
                let c = Credential {
                    cookies: wbi_client.get_cookies()?,
                    refresh_token: poll.refresh_token,
                };
                QRCodeLoginState::Success(c)
            }
            86038 => QRCodeLoginState::QRCodeExpired,
            86090 => QRCodeLoginState::WaitConfirm,
            86101 => QRCodeLoginState::WaitScan,
            _ => return Err(BError::from_json_err("Invalid login state code found.")),
        };
        Ok(state)
    }
}

#[cfg(test)]
mod test {
    use crate::wbi_client::WbiClient;

    use super::QRCodeLogin;
    #[tokio::test]
    async fn test_get_info() {
        let client = WbiClient::builder().build().await.unwrap();
        let _info = QRCodeLogin::get_login_info(&client).await.unwrap();
        let _qrcode = _info.get_login_qrcode().unwrap();
    }
}
