use crate::{
    error::TusError,
    tus::{http::TusHttpMethod, ops::TusOp, upload_meta::UploadMeta, TusServerInfo},
};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client as RequestClient, Request,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
    str::FromStr,
};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientOptions {
    /// chunksize to use for uploading very large files
    ///
    /// Defaults to 6MB
    pub chunksize: usize,
}

impl ClientOptions {
    pub fn new(chunksize: usize) -> Self {
        Self { chunksize }
    }

    pub fn default() -> Self {
        Self {
            chunksize: 6 * 1024 * 1024, // 6MB
        }
    }
}

pub struct Client {
    client: RequestClient,
    options: ClientOptions,
}

impl Client {
    /// Create a new TUS Client
    pub fn new(options: ClientOptions) -> Self {
        let client = RequestClient::new();
        Self { client, options }
    }

    /// Run TUS Operations
    ///
    /// Each operation implementation handles deriving the building blocks for creating the http
    /// request
    ///
    /// Returns: `UploadMeta`
    async fn run(
        &self,
        op: TusOp,
        metadata: &UploadMeta,
        body: Option<&[u8]>,
    ) -> Result<UploadMeta, TusError> {
        let headers = op.headers(metadata)?;
        let url = op.url_for_meta(metadata);
        let request = self.make_request(&url, op.method(), headers, body)?;
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| TusError::RequestError(format!("{e}")))?;
        match response.status().as_u16() {
            200..=299 => {
                // Happy path
                op.handle_response(response, metadata)
            }
            400 => Err(TusError::BadRequest(format!(
                "{}",
                response.text().await.unwrap_or("".to_string())
            ))),
            404 => Err(TusError::NotFoundError),
            409 => Err(TusError::WrongUploadOffsetError),
            413 => Err(TusError::FileTooLarge),
            460 => Err(TusError::ChecksumMismatch),
            _ => Err(TusError::UnexpectedStatusCode(
                response.status().as_u16().into(),
                response.text().await.unwrap_or("".to_string()),
            )),
        }
    }

    fn make_request(
        &self,
        url: &Url,
        method: TusHttpMethod,
        headers: HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> Result<Request, TusError> {
        let mut map = HeaderMap::new();
        for (k, v) in headers.iter() {
            let name = HeaderName::from_str(&k).map_err(|_| TusError::InvalidHeader(k.clone()))?;
            let value =
                HeaderValue::from_str(&v).map_err(|_| TusError::InvalidHeaderValue(v.clone()))?;
            map.insert(name, value);
        }
        let mut request = self
            .client
            .request(method.to_method(), url.clone())
            .headers(map);
        if let Some(body) = body {
            request = request.body(Vec::from(body));
        }
        request
            .build()
            .map_err(|e| TusError::RequestError(format!("{e}")))
    }

    /// Get the server info
    pub async fn get_server_info(&self, url: &Url) -> Result<TusServerInfo, TusError> {
        let headers = HashMap::<String, String>::new();
        let request = self.make_request(url, TusHttpMethod::Options, headers, None)?;
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| TusError::ReqwestError(e))?;

        match response.status().as_u16() {
            204 | 200 => {
                // 204 No Content or 200 OK
                Ok(response.headers().to_owned().into())
            }
            _ => Err(TusError::RequestError(format!(
                "Error code: {}, Text: {}",
                response.status(),
                response.text().await.unwrap_or("".to_string())
            ))),
        }
    }

    /// Create a resource on the server to upload a file
    pub async fn create(
        &self,
        file: &PathBuf,
        host: &Url,
        metadata: Option<HashMap<String, String>>,
        custom_headers: Option<HashMap<String, String>>,
    ) -> Result<UploadMeta, TusError> {
        // Create initial metadata
        let meta = UploadMeta::new(file.clone(), host.clone(), None, metadata, custom_headers)?;

        // ** create resource on server **
        let meta = self.run(TusOp::Create, &meta, None).await?;
        Ok(meta)
    }

    /// Get offset for an existing resource
    pub async fn get_offset(&self, meta: &UploadMeta) -> Result<UploadMeta, TusError> {
        self.run(TusOp::GetOffset, &meta, None).await
    }

    /// Resume an upload
    pub async fn resume(&self, meta: &UploadMeta) -> Result<UploadMeta, TusError> {
        // # Upload file
        //
        // From Protocol:
        //
        // > The Client SHOULD send all the remaining bytes of an upload in a single PATCH
        // > request, but MAY also use multiple small requests successively
        // > for scenarios where this is desirable. One example for these
        // > situations is when the Checksum extension is used.

        let file = File::open(&meta.file_path)?;
        let mut reader = BufReader::new(&file);
        let mut buffer = vec![0; self.options.chunksize];
        let mut meta = meta.clone();

        reader.seek(SeekFrom::Start(meta.status.bytes_uploaded as u64))?;

        // TODO: if upload fails, return upload metadata to resume with later
        // likely need different function return type
        loop {
            let bytes_count = reader.read(&mut buffer)?;
            if bytes_count == 0 {
                return Err(TusError::FileReadError(
                    "Zero bytes read from file".to_string(),
                ));
            }
            let body = Some(&buffer[..bytes_count]);
            meta = self.run(TusOp::Upload, &meta, body).await?;
            if meta.upload_complete() {
                break;
            }
        }
        Ok(meta)
    }

    /// Upload a file
    ///
    /// Creates a resource on server and uploads the file
    pub async fn upload(
        &self,
        file: &PathBuf,
        host: &Url,
        metadata: Option<HashMap<String, String>>,
        custom_headers: Option<HashMap<String, String>>,
    ) -> Result<UploadMeta, TusError> {
        let meta = self.create(file, host, metadata, custom_headers).await?;
        self.resume(&meta).await
    }

    /// Terminate upload and delete file
    pub async fn terminate(&self, meta: &UploadMeta) -> Result<(), TusError> {
        let _result = self.run(TusOp::Terminate, meta, None).await;
        Ok(())
    }
}
