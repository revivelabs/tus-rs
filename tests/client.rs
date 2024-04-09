use std::io::Write;
use tus_rs::client::*;
use tempfile::NamedTempFile;
use url::Url;

const TUS_ENDPOINT: &str = "http://127.0.0.1:8080/files/";

fn create_temp_file() -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    let buffer: Vec<u8> = (0..(1024 * 1024)).map(|_| rand::random::<u8>()).collect();
    for _ in 0..20 {
        temp_file.write_all(&buffer[..]).unwrap();
    }
    temp_file
}

#[tokio::test]
async fn should_get_server_info() {
    let url = Url::parse(TUS_ENDPOINT).unwrap();
    let client = Client::new();
    let result = client.get_server_info(&url).await;
    dbg!(&result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_create_file() {
    let temp_file = create_temp_file();
    let path = temp_file.path().into();
    let client = Client::new();
    let host = Url::parse(TUS_ENDPOINT).unwrap();
    let result = client.upload(&path, &host, None, None).await;
    dbg!(&result);
    assert!(result.is_ok());
}
