use bilibili_api::{login::QRCodeLogin, login::QRCodeLoginState, wbi_client::WbiClient};
use tokio::time::{sleep, Duration};
const COMMON_DIR: &str = "./examples/saves/";

#[tokio::main]
async fn main() {
    let client = WbiClient::builder().build().await.unwrap();
    let login = QRCodeLogin::get_login_info(&client).await.unwrap();
    let code = login.get_login_qrcode().unwrap();
    let string = code
        .render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("{}", string);
    let cred = loop {
        let state = login.poll_login_state(&client).await.unwrap();
        match state {
            QRCodeLoginState::Success(cred) => break cred,
            QRCodeLoginState::QRCodeExpired => {
                eprintln!("QRcode expired!");
                return;
            }
            QRCodeLoginState::WaitConfirm => {
                println!("Wait Confirm");
            }
            QRCodeLoginState::WaitScan => {
                println!("Wait Scan");
            }
        }
        sleep(Duration::from_secs(10)).await;
    };
    println!("Login success");
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(std::path::Path::new(COMMON_DIR).join("cred.json"))
        .unwrap();
    cred.save_json(&mut f).unwrap();
}
