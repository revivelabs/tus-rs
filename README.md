# `tus-rs`

Rust implementation of TUS protocol for resumable uploads.

Heavily influenced by:

- [`tus_client`](https://github.com/jonstodle/tus_client)
- [`TusKit`](https://github.com/tus/TusKit)

# Usage

Create a resource

```rust
let path = PathBuf::from_str("/path/to/file")?;
let client = Client::new();
let host = Url::parse(TUS_ENDPOINT).unwrap();
let upload_metadata = client.create(&path, &host, None, None).await?;
```

Resume the upload

```rust
let client = Client::new();
let host = Url::parse(TUS_ENDPOINT).unwrap();
let upload_metadata = // get metadata from the `create` function, or disk/memory
let upload_metadata = client.resume(&upload_metadata).await?;
```

Create and upload a file same time

```rust
let path = PathBuf::from_str("/path/to/file")?;
let client = Client::new();
let host = Url::parse(TUS_ENDPOINT).unwrap();
let extra_metadata = Some(HashMap::<String,String>::new());
let custom_headers = None;
let result = client.upload(&path, &host, extra_metadata, custom_headers).await;
```
