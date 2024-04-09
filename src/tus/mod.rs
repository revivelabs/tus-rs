pub mod errors;
pub mod headers;
pub mod http;
pub mod ops;
pub mod upload_meta;

use std::str::FromStr;

use reqwest::header::HeaderMap;
use serde;
use serde::{Deserialize, Serialize};

use crate::error::TusError;
use crate::tus::headers::TusHeaders;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct TusServerInfo {
    pub version: Option<String>,
    pub max_size: Option<usize>,
    pub extensions: Vec<TusExtension>,
    pub supported_versions: Vec<String>,
    pub supported_checksum_algorithms: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TusExtension {
    Creation,
    CreationWithUpload,
    Termination,
    Expiration,
    Concatenation,
    CreationDeferLength,
}

impl FromStr for TusExtension {
    type Err = TusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{s}\""))
            .map_err(|_| TusError::StringParseError(format!("Invalid TusExtension String: {s}")))
    }
}

impl From<TusHeaders> for TusServerInfo {
    fn from(headers: TusHeaders) -> Self {
        let version: Option<String> = headers.version;
        let max_size: Option<usize> = headers.max_size;
        let extensions: Vec<TusExtension> = headers.extensions.unwrap_or_default();
        let supported_versions: Vec<String> = headers.supported_versions.unwrap_or_default();
        let supported_checksum_algorithms: Option<Vec<String>> = headers.checksum_algorithms;
        return Self {
            version,
            max_size,
            extensions,
            supported_versions,
            supported_checksum_algorithms,
        };
    }
}

impl From<HeaderMap> for TusServerInfo {
    fn from(value: HeaderMap) -> Self {
        let headers: TusHeaders = value.into();
        headers.into()
    }
}
