use reqwest::header::HeaderValue;
use std::collections::HashMap;

/// Indicates a byte offset withing a resource.
pub const UPLOAD_OFFSET: &'static str = "Upload-Offset";

/// Indicates the size of the entire upload in bytes.
pub const UPLOAD_LENGTH: &'static str = "Upload-Length";

/// A comma-separated list of protocol versions supported by the server.
pub const TUS_VERSION: &'static str = "Tus-Version";

/// The version of the protocol used by the client or the server.
pub const TUS_RESUMABLE: &'static str = "Tus-Resumable";

/// A comma-separated list of the extensions supported by the server.
pub const TUS_EXTENSION: &'static str = "Tus-Extension";

/// Integer indicating the maximum allowed size of an entire upload in bytes.
pub const TUS_MAX_SIZE: &'static str = "Tus-Max-Size";

///
pub const TUS_CHECKSUM_ALGO: &'static str = "Tus-Checksum-Algorithm";

/// Use this header if its environment does not support the PATCH or DELETE methods.
pub const X_HTTP_METHOD_OVERRIDE: &'static str = "X-HTTP-Method-Override";

/// Use this header if its environment does not support the PATCH or DELETE methods.
pub const CONTENT_TYPE: &'static str = "Content-Type";

/// Use this header if its environment does not support the PATCH or DELETE methods.
//pub const UPLOAD_DEFER_LENGTH: &'static str = "upload-defer-length";

/// Use this header if its environment does not support the PATCH or DELETE methods.
pub const UPLOAD_METADATA: &'static str = "Upload-Metadata";

/// Use this header when creating an upload to get the location of the upload on the server
pub const TUS_LOCATION: &'static str = "Location";

/// An alias for `HashMap<String, String>`, which represents a set of HTTP headers and their values.
pub type Headers = HashMap<String, String>;

pub fn default_headers() -> Headers {
    let mut map = Headers::new();
    map.insert(String::from(TUS_RESUMABLE), String::from("1.0.0"));
    map
}

// struct TusHeaders {
//     pub offset: Option<usize>,
//     pub upload_length: Option<usize>,
//     pub version: Option<String>,
//     pub supported_versions: Option<Vec<String>>,
//     pub resumable: Option<String>,
//     pub extensions: Option<Vec<TusOp>>,
//     pub max_size: Option<usize>,
//     pub checksum_algorithms: Option<Vec<String>>,
//     pub upload_metadata: Option<HashMap<String, String>>,
//     pub location: Option<String>,
// }
//
// impl From<HeaderMap> for TusHeaders {
//     fn from(value: HeaderMap) -> Self {
//         let version: Option<String> = value
//             .get(TUS_RESUMABLE)
//             .map_or(None, |v| Some(v.to_str().unwrap().to_string()));
//
//         let max_size: Option<usize> = value.get(TUS_MAX_SIZE).map_or(None, |v| {
//             v.to_str()
//                 .unwrap()
//                 .to_string()
//                 .parse::<usize>()
//                 .unwrap()
//                 .into()
//         });
//         let extensions = match value.get(TUS_EXTENSION) {
//             Some(value) => Some(
//                 value
//                     .to_str()
//                     .unwrap()
//                     .split(',')
//                     .map(str::parse)
//                     .filter(Result::is_ok)
//                     .map(Result::unwrap)
//                     .collect::<Vec<TusOp>>(),
//             ),
//             _ => None,
//         };
//         let supported_versions: Option<Vec<String>> = value.get(TUS_VERSION).map_or(None, |v| {
//             let versions = v
//                 .to_str()
//                 .unwrap()
//                 .split(',')
//                 .map(String::from)
//                 .collect::<Vec<String>>();
//             Some(versions)
//         });
//
//         let checksum_algorithms: Option<Vec<String>> = match value.get(TUS_CHECKSUM_ALGO) {
//             Some(value) => Some(
//                 value
//                     .to_str()
//                     .unwrap()
//                     .split(',')
//                     .map(String::from)
//                     .collect::<Vec<String>>(),
//             ),
//             _ => None,
//         };
//         let offset = value.get(UPLOAD_OFFSET).map_or(None, |v| {
//             str::parse::<usize>(&v.to_str().unwrap_or("")).ok()
//         });
//         let upload_length = value.get(UPLOAD_LENGTH).map_or(None, |v| {
//             str::parse::<usize>(&v.to_str().unwrap_or("")).ok()
//         });
//
//         let resumable = value
//             .get(TUS_RESUMABLE)
//             .map_or(None, |v| v.to_string().into());
//
//         Self {
//             offset,
//             upload_length,
//             version,
//             supported_versions,
//             resumable,
//             extensions,
//             max_size,
//             checksum_algorithms,
//             upload_metadata: todo!(),
//             location: todo!(),
//         }
//     }
// }

/// Additional conversion methods for `HeaderValue`.
pub trait HeaderValueExt {
    fn to_string(&self) -> String;
}

impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> String {
        self.to_str().unwrap_or_default().to_string()
    }
}
