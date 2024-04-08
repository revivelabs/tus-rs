use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
    str::FromStr,
};

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client as RequestClient, Request,
};
use url::Url;

use crate::{
    error::TusError,
    tus::{http::TusHttpMethod, upload_meta::UploadMeta, TusOp, TusServerInfo},
};

pub struct Client {
    pub use_method_override: bool,
    client: RequestClient,
}

impl Client {
    /// Create an upload with metadata
    ///
    /// Returns: `UploadMeta` with the `remote_dest` value set to the location on the server
    async fn run(
        &self,
        op: TusOp,
        metadata: &UploadMeta,
        body: Option<&[u8]>,
    ) -> Result<UploadMeta, TusError> {
        let headers = op.headers(metadata)?;
        let upload_url = metadata
            .upload_url()
            .map_err(|_| TusError::MissingUploadUrl)?;
        let request = self.make_request(&upload_url, op.method(), headers, body)?;
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|_| TusError::RequestError)?;
        match response.status().as_u16() {
            200..=299 => {
                // this is what we expect
                op.handle_response(response, metadata)
            }
            404 => Err(TusError::NotFoundError),
            409 => Err(TusError::WrongUploadOffsetError),
            413 => Err(TusError::FileTooLarge),
            _ => Err(TusError::UnexpectedStatusCode(
                response.status().as_u16().into(),
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
        let body = body.map_or(Vec::new(), |v| Vec::from(v));
        let request = self
            .client
            .request(method.to_method(), url.clone())
            .headers(map)
            .body(body);
        request.build().map_err(|_| TusError::RequestError)
    }

    /// Get the server info
    pub async fn get_server_info(&self, url: &Url) -> Result<TusServerInfo, TusError> {
        let headers = HashMap::<String, String>::new();
        let request = self.make_request(url, TusHttpMethod::Options, headers, None)?;
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|_| TusError::RequestError)?;
        match response.status().as_u16() {
            204 | 200 => {
                // 204 No Content or 200 OK
                Ok(response.headers().to_owned().into())
            }
            _ => Err(TusError::RequestError),
        }
    }

    /// Upload a file
    pub async fn upload(
        &self,
        file: &PathBuf,
        host: Url,
        remote_dest: PathBuf,
        metadata: Option<HashMap<String, String>>,
        custom_headers: Option<HashMap<String, String>>,
    ) -> Result<(), TusError> {
        // Create initial metadata
        let meta = UploadMeta::new(
            file.clone(),
            host,
            remote_dest,
            None,
            metadata,
            custom_headers,
            None,
        )?;

        // ** create resource on server **
        let meta = self.run(TusOp::Creation, &meta, None).await?;

        // ** upload file **
        //
        // From Protocol:
        //
        // The Client SHOULD send all the remaining bytes of an upload in a single PATCH
        // request, but MAY also use multiple small requests successively
        // for scenarios where this is desirable. One example for these
        // situations is when the Checksum extension is used.

        let file = File::open(&meta.file_path)?;
        let mut reader = BufReader::new(&file);
        let mut buffer = vec![0; meta.chunksize];

        reader.seek(SeekFrom::Start(meta.status.bytes_uploaded as u64))?;

        // TODO: if upload fails, persist upload meta data to resume with later

        loop {
            let bytes_count = reader.read(&mut buffer)?;
            if bytes_count == 0 {
                return Err(TusError::FileReadError(
                    "Zero bytes read from file".to_string(),
                ));
            }
            let body = Some(&buffer[..bytes_count]);

            let meta = self.run(TusOp::Upload, &meta, body).await?;
            if meta.status.bytes_uploaded >= meta.status.size {
                break;
            }
        }
        Ok(())
    }
}
