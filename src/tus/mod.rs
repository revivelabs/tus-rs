pub mod errors;
pub mod headers;
pub mod http;
pub mod upload_meta;

use crate::{error::TusError, tus};
use reqwest::header::HeaderMap;
use reqwest::Response;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use self::headers::UPLOAD_OFFSET;
use self::http::TusHttpMethod;
use self::upload_meta::UploadMeta;

// use self::http::{Headers, HttpRequest};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TusOp {
    // ************
    // Core
    // ************
    /// Determines the offset at which the upload should be continued
    ///
    /// **Assumes URL for upload has already been created by the Creation extension**
    ///
    /// _Returns an error if not_
    ///
    /// Returns the `UploadMeta` with the `bytes_uploaded` updated with the
    /// returned offset
    GetOffset,

    /// Resume upload
    Upload,

    // ************
    // Extensions
    // ************
    /// The server supports creating files.
    Creation,
    //// The server supports setting expiration time on files and uploads.
    Expiration,
    /// The server supports verifying checksums of uploaded chunks.
    Checksum,
    /// The server supports deleting files.
    Termination,
    /// The server supports parallel uploads of a single file.
    Concatenation,
}

impl From<TusOp> for TusHttpMethod {
    fn from(extension: TusOp) -> Self {
        extension.method()
    }
}

impl TusOp {
    pub fn method(&self) -> TusHttpMethod {
        match self {
            TusOp::GetOffset => TusHttpMethod::Head,

            // all patch requests must contain
            // "Content-Type": "application/offset+octet-stream"
            TusOp::Upload => TusHttpMethod::Patch,
            TusOp::Creation => TusHttpMethod::Post,
            TusOp::Expiration => TusHttpMethod::Post,
            TusOp::Checksum => TusHttpMethod::Post,
            TusOp::Termination => TusHttpMethod::Post,
            TusOp::Concatenation => TusHttpMethod::Post,
        }
    }

    pub fn headers(&self, metadata: &UploadMeta) -> Result<HashMap<String, String>, TusError> {
        let mut headers = tus::headers::default_headers();
        let data = metadata.data64()?;
        headers.insert(tus::headers::UPLOAD_METADATA.to_owned(), data);
        if let Some(custom_headers) = &metadata.custom_headers {
            headers.extend(custom_headers.clone());
        }
        match self {
            TusOp::Creation => {
                headers.insert(
                    tus::headers::UPLOAD_LENGTH.to_owned(),
                    format!("{}", metadata.status.size),
                );
            }
            TusOp::Upload => {
                headers.insert(
                    tus::headers::CONTENT_TYPE.to_owned(),
                    "application/offset+octet-stream".to_string(),
                );
                headers.insert(
                    tus::headers::UPLOAD_LENGTH.to_owned(),
                    format!("{}", metadata.status.size),
                );
                headers.insert(
                    tus::headers::UPLOAD_OFFSET.to_owned(),
                    format!("{}", metadata.status.bytes_uploaded),
                );
            }
            _ => {}
        }
        Ok(headers)
    }

    pub fn handle_response(
        &self,
        response: Response,
        metadata: &UploadMeta,
    ) -> Result<UploadMeta, TusError> {
        match self {
            TusOp::Creation => {
                let remote_dest = response
                    .headers()
                    .get(tus::headers::TUS_LOCATION)
                    .ok_or(TusError::MissingHeader(
                        tus::headers::TUS_LOCATION.to_owned(),
                    ))?
                    .to_str()
                    .map_err(|_| TusError::MissingHeader(tus::headers::TUS_LOCATION.to_owned()))?;
                let remote_dest = PathBuf::from_str(remote_dest)
                    .map_err(|_| TusError::MissingHeader(tus::headers::TUS_LOCATION.to_owned()))?
                    .into();
                Ok(UploadMeta {
                    remote_dest,
                    ..metadata.clone()
                })
            }
            TusOp::GetOffset => {
                let offset = response
                    .headers()
                    .get(tus::headers::UPLOAD_OFFSET)
                    .ok_or(TusError::RequestError)?
                    .to_str()?;
                let offset = str::parse::<usize>(offset)?;
                Ok(metadata.with_bytes_uploaded(offset))
            }
            TusOp::Upload => {
                // let headers: HashMap<String,String> = response
                //     .headers()
                //     .iter()
                //     .map(|(k, v)| (format!("{k}"), format!("{}", v.to_string()))).collect();
                let offset = response
                    .headers()
                    .get(UPLOAD_OFFSET)
                    .map_or(None, |v| {
                        str::parse::<usize>(&v.to_str().unwrap_or("")).ok()
                    })
                    .ok_or(TusError::MissingHeader(
                        tus::headers::UPLOAD_OFFSET.to_owned(),
                    ))?;
                Ok(metadata.with_bytes_uploaded(offset))
            }
            _ => Ok(metadata.clone()),
        }
    }
}

impl FromStr for TusOp {
    type Err = TusError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&value).map_err(|_| TusError::SerdeError)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadStatus {
    /// total range uploaded
    pub bytes_uploaded: usize,

    /// total size of file in bytes
    pub size: usize,
}

impl UploadStatus {
    pub fn new(size: usize, bytes_uploaded: Option<usize>) -> Self {
        UploadStatus {
            size,
            bytes_uploaded: bytes_uploaded.unwrap_or(0),
        }
    }
}

// pub struct UploadStatus {
//     length: usize,
//     offset: usize,
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct TusServerInfo {
    pub version: Option<String>,
    pub max_size: Option<usize>,
    pub extensions: Option<Vec<TusOp>>,
    pub supported_versions: Vec<String>,
    pub supported_checksum_algorithms: Option<Vec<String>>,
}

// impl Into<HeaderMap> for HashMap<String, String> {
//     fn into(self) -> HeaderMap {
//         let mut headers = Self::new();
//         for (key, value) in map {
//             headers.insert(HeaderName::from_str(&key), value.parse().unwrap());
//         }
//         headers
//     }
// }

impl From<HeaderMap> for TusServerInfo {
    fn from(value: HeaderMap) -> Self {
        let version: Option<String> = value
            .get(headers::TUS_RESUMABLE)
            .map_or(None, |v| Some(v.to_str().unwrap().to_string()));

        let max_size: Option<usize> = value.get(headers::TUS_MAX_SIZE).map_or(None, |v| {
            v.to_str()
                .unwrap()
                .to_string()
                .parse::<usize>()
                .unwrap()
                .into()
        });
        let extensions = match value.get(headers::TUS_EXTENSION) {
            Some(value) => Some(
                value
                    .to_str()
                    .unwrap()
                    .split(',')
                    .map(str::parse)
                    .filter(Result::is_ok)
                    .map(Result::unwrap)
                    .collect::<Vec<TusOp>>(),
            ),
            _ => None,
        };
        let supported_versions: Vec<String> =
            value.get(headers::TUS_VERSION).map_or(Vec::new(), |v| {
                v.to_str()
                    .unwrap()
                    .split(',')
                    .map(String::from)
                    .collect::<Vec<String>>()
            });

        let supported_checksum_algorithms: Option<Vec<String>> =
            match value.get(headers::TUS_CHECKSUM_ALGO) {
                Some(value) => Some(
                    value
                        .to_str()
                        .unwrap()
                        .split(',')
                        .map(String::from)
                        .collect::<Vec<String>>(),
                ),
                _ => None,
            };

        return Self {
            version,
            max_size,
            extensions,
            supported_versions,
            supported_checksum_algorithms,
        };
    }
}

// fn build_request<'a>(extension: TusOp) -> Result<HttpRequest<'a>, Error> {}
