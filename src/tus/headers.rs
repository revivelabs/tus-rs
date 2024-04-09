use base64::Engine;
use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;

use super::{FromStr, TusExtension};

/// Indicates a byte offset withing a resource.
pub const UPLOAD_OFFSET: &'static str = "upload-offset";

/// Indicates the size of the entire upload in bytes.
pub const UPLOAD_LENGTH: &'static str = "upload-length";

/// A comma-separated list of protocol versions supported by the server.
pub const TUS_VERSION: &'static str = "tus-version";

/// The version of the protocol used by the client or the server.
pub const TUS_RESUMABLE: &'static str = "tus-resumable";

/// A comma-separated list of the extensions supported by the server.
pub const TUS_EXTENSION: &'static str = "tus-extension";

/// Integer indicating the maximum allowed size of an entire upload in bytes.
pub const TUS_MAX_SIZE: &'static str = "tus-max-size";

///
pub const TUS_CHECKSUM_ALGO: &'static str = "tus-checksum-algorithm";

/// Use this header if its environment does not support the PATCH or DELETE methods.
pub const X_HTTP_METHOD_OVERRIDE: &'static str = "x-http-method-override";

/// Use this header if its environment does not support the PATCH or DELETE methods.
pub const CONTENT_TYPE: &'static str = "content-type";

/// Use this header if its environment does not support the PATCH or DELETE methods.
pub const UPLOAD_DEFER_LENGTH: &'static str = "upload-defer-length";

/// Use this header if its environment does not support the PATCH or DELETE methods.
pub const UPLOAD_METADATA: &'static str = "upload-metadata";

/// Use this header when creating an upload to get the location of the upload on the server
pub const TUS_LOCATION: &'static str = "location";

/// An alias for `HashMap<String, String>`, which represents a set of HTTP headers and their values.
pub type Headers = HashMap<String, String>;

pub fn default_headers() -> Headers {
    let mut map = Headers::new();
    map.insert(String::from(TUS_RESUMABLE), String::from("1.0.0"));
    map
}

pub struct TusHeaders {
    pub offset: Option<usize>,
    pub upload_length: Option<usize>,
    pub version: Option<String>,
    pub supported_versions: Option<Vec<String>>,
    pub resumable: Option<String>,
    pub extensions: Option<Vec<TusExtension>>,
    pub max_size: Option<usize>,
    pub checksum_algorithms: Option<Vec<String>>,
    pub upload_metadata: Option<HashMap<String, String>>,
    pub location: Option<String>,
}

impl From<HeaderMap> for TusHeaders {
    fn from(value: HeaderMap) -> Self {
        let headers: HashMap<String, String> = value
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let version: Option<String> = headers.get(TUS_RESUMABLE).map(|v| v.to_string());
        let max_size: Option<usize> = headers
            .get(TUS_MAX_SIZE)
            .map(|v| v.parse::<usize>().unwrap().into());
        let extensions: Option<Vec<TusExtension>> = headers.get(TUS_EXTENSION).map(|string| {
            string
                .split(',')
                .filter_map(|s| TusExtension::from_str(s).ok())
                .collect()
        });
        let supported_versions: Option<Vec<String>> = headers
            .get(TUS_VERSION)
            .map(|v| v.split(',').map(String::from).collect::<Vec<String>>());
        let checksum_algorithms: Option<Vec<String>> = headers
            .get(TUS_CHECKSUM_ALGO)
            .map(|value| value.split(',').map(String::from).collect::<Vec<String>>());
        let offset = headers
            .get(UPLOAD_OFFSET)
            .map_or(None, |v| str::parse::<usize>(&v).ok());
        let upload_length = headers
            .get(UPLOAD_LENGTH)
            .map_or(None, |v| str::parse::<usize>(&v).ok());
        let resumable = headers.get(TUS_RESUMABLE).map(|s| s.to_owned());
        let location = headers.get(TUS_LOCATION).map(|s| s.to_owned());
        let upload_metadata = headers
            .get(UPLOAD_METADATA)
            .map_or(None, |list| {
                base64::engine::general_purpose::STANDARD.decode(list).ok()
            })
            .map(|decoded| {
                String::from_utf8(decoded).unwrap().split(";").fold(
                    HashMap::new(),
                    |mut acc, key_val| {
                        let mut parts = key_val.splitn(2, ':');
                        if let Some(key) = parts.next() {
                            acc.insert(
                                String::from(key),
                                String::from(parts.next().unwrap_or_default()),
                            );
                        }
                        acc
                    },
                )
            });

        Self {
            offset,
            upload_length,
            version,
            supported_versions,
            resumable,
            extensions,
            max_size,
            checksum_algorithms,
            upload_metadata,
            location,
        }
    }
}

/// Additional conversion methods for `HeaderValue`.
pub trait HeaderValueExt {
    fn to_string(&self) -> String;
}

impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> String {
        self.to_str().unwrap_or_default().to_string()
    }
}
