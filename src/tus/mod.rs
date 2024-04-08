pub mod errors;
pub mod headers;
pub mod http;
pub mod ops;
pub mod upload_meta;

pub use ops::*;
use reqwest::header::HeaderMap;
use serde;
use serde::{Deserialize, Serialize};

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

