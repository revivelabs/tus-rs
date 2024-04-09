use crate::{error::TusError, tus};
use reqwest::Response;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::metadata;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

use super::headers::TusHeaders;
use super::http::TusHttpMethod;
use super::upload_meta::UploadMeta;

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

    /// The server supports deleting files.
    Termination,
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
            TusOp::Creation => TusHttpMethod::Post, // empty post request
            // TusOp::Expiration => TusHttpMethod::Patch, //
            TusOp::Termination => TusHttpMethod::Delete,
            // TusOp::Concatenation => TusHttpMethod::Post,
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
                // headers.insert(
                //     tus::headers::UPLOAD_LENGTH.to_owned(),
                //     format!("{}", metadata.status.size),
                // );
                headers.insert(
                    tus::headers::UPLOAD_OFFSET.to_owned(),
                    format!("{}", metadata.status.bytes_uploaded),
                );
                dbg!(&metadata);
            }
            _ => {}
        }
        Ok(headers)
    }

    pub fn url_for_meta(&self, metadata: &UploadMeta) -> Url {
        match self {
            TusOp::Upload => metadata
                .remote_url
                .clone()
                .unwrap_or(metadata.upload_host.clone())
                .clone(),
            _ => metadata.upload_host.clone(),
        }
    }

    pub fn handle_response(
        &self,
        response: Response,
        metadata: &UploadMeta,
    ) -> Result<UploadMeta, TusError> {
        let headers: TusHeaders = response.headers().clone().into();
        match self {
            TusOp::Creation => {
                let remote_dest = headers.location.ok_or(TusError::MissingHeader(
                    tus::headers::TUS_LOCATION.to_owned(),
                ))?;
                metadata.with_remote_dest(remote_dest)
            }
            TusOp::GetOffset => {
                let offset = headers
                    .offset
                    .ok_or(TusError::RequestError("Missing offset".to_string()))?;
                Ok(metadata.with_bytes_uploaded(offset))
            }
            TusOp::Upload => {
                let offset = headers
                    .offset
                    .ok_or(TusError::RequestError("Missing offset".to_string()))?;
                dbg!(offset);
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
