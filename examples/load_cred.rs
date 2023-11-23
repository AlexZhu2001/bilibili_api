use bilibili_api::{login::Credential, wbi_client::WbiClient};
const COMMON_DIR: &str = "./examples/saves/";

#[tokio::main]
async fn main() {
    let f = std::fs::File::open(std::path::Path::new(COMMON_DIR).join("cred.json")).unwrap();
    let rdr = std::io::BufReader::new(&f);
    let mut cred = Credential::load_json(rdr).unwrap();
    let _client = WbiClient::builder()
        .with_credential(&mut cred)
        .await
        .unwrap()
        .build()
        .await
        .unwrap();
    drop(f);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(std::path::Path::new(COMMON_DIR).join("cred.json"))
        .unwrap();
    cred.save_json(&mut f).unwrap();
}
