use std::io::Write;
use tempfile::NamedTempFile;
use tus_rs::client::*;
use url::Url;

// TODO: add github actions to test using the tusd docker image:
// https://tus.github.io/tusd/getting-started/installation/#docker-container

const TUS_ENDPOINT: &str = "http://127.0.0.1:8080/files/";

fn create_temp_file(size: usize) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    let buffer: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
    for _ in 0..20 {
        temp_file.write_all(&buffer[..]).unwrap();
    }
    temp_file
}

#[tokio::test]
async fn should_get_server_info() {
    let url = Url::parse(TUS_ENDPOINT).unwrap();
    let client = Client::new(ClientOptions::default());
    let result = client.get_server_info(&url).await;
    dbg!(&result);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.extensions.is_empty());
}

#[tokio::test]
async fn should_create_file() {
    let temp_file = create_temp_file(128);
    let path = temp_file.path().into();
    let client = Client::new(ClientOptions::default());
    let host = Url::parse(TUS_ENDPOINT).unwrap();
    let result = client.create(&path, &host, None, None).await;
    dbg!(&result);
    assert!(result.is_ok());
}

// #[tokio::test]
// async fn should_resume_file() {
//     todo!()
// }

#[tokio::test]
async fn should_create_and_upload_file() {
    let temp_file = create_temp_file(1024 * 100);
    let path = temp_file.path().into();
    let client = Client::new(ClientOptions::default());
    let host = Url::parse(TUS_ENDPOINT).unwrap();
    let result = client.upload(&path, &host, None, None).await;
    dbg!(&result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_create_and_terminate_file() {
    let temp_file = create_temp_file(1024 * 100);
    let path = temp_file.path().into();
    let client = Client::new(ClientOptions::default());
    let host = Url::parse(TUS_ENDPOINT).unwrap();
    let result = client.create(&path, &host, None, None).await;
    dbg!(&result);
    assert!(result.is_ok());
    let meta = result.unwrap();
    let result = client.terminate(&meta).await;
    dbg!(&result);
    assert!(result.is_ok());
}
