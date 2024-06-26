use crate::error::TusError;
use base64::Engine;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

use super::UploadStatus;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadMeta {
    /// upload_host for the file - e.g. "http://www.tusserver.com"
    pub upload_host: Url,

    /// absolute local path to file
    pub file_path: PathBuf,

    /// path on the server to the file creation request is sent
    ///
    /// set by the server
    ///
    /// e.g. "[upload_host]/path/to/file/on/server"
    pub remote_url: Option<Url>,

    /// Status of the upload
    pub status: UploadStatus,

    /// TUS version associated with file
    pub version: String,

    /// any extra meta data to include in the upload
    ///  Will be added as base64 key:value encoded pairs
    ///  the UPLOAD_METADATA header value
    pub extra_meta: Option<HashMap<String, String>>,

    /// File type
    pub mime_type: Option<String>,

    /// any custom headers to add to the requests
    pub custom_headers: Option<HashMap<String, String>>,

    /// number of times upload attempted/failed
    pub error_count: usize,
}

/// Validates the filename of `file_path` and checks to make sure it is well-formatted
/// i.e.
/// - not a directory
/// - file exists
/// - filename != ""
/// - filename != "/"
fn validate_path(file_path: &PathBuf) -> Result<(), TusError> {
    if !file_path.exists() {
        return Err(TusError::FileReadError("File not found".to_string()));
    }
    if file_path.is_dir() {
        return Err(TusError::FileReadError("Cannot be a directory".to_string()));
    }
    let filename = file_path.file_name().ok_or(TusError::EmptyFilename)?;
    let filename = filename
        .to_str()
        .ok_or(TusError::InvalidFilename(
            "Unable to convert to string".to_string(),
        ))?
        .to_string();
    if filename == "/".to_string() {
        return Err(TusError::InvalidFilename(
            "Filename cannot be '/'".to_string(),
        ));
    }
    Ok(())
}

impl UploadMeta {
    pub fn new(
        file_path: PathBuf,
        upload_host: Url,
        bytes_uploaded: Option<usize>,
        extra_meta: Option<HashMap<String, String>>,
        custom_headers: Option<HashMap<String, String>>,
    ) -> Result<Self, TusError> {
        validate_path(&file_path)?;
        let file_meta = file_path.metadata()?;
        let size: usize = file_meta.len() as usize;
        let status = UploadStatus::new(size, bytes_uploaded);
        let meta = UploadMeta {
            file_path,
            upload_host,
            extra_meta,
            custom_headers,
            status,
            error_count: 0,
            version: "1".to_string(), // Version of TUS protocol
            remote_url: None,
            // with value present
            mime_type: None, // TODO: Set this based on file extension?
        };

        Ok(meta)
    }

    // /// Convenience getter to get the filename of the filepath as a string
    // pub fn filename(&self) -> String {
    //     self.file_path
    //         .file_name()
    //         .ok_or(TusError::EmptyFilename)
    //         .unwrap()
    //         .to_str()
    //         .unwrap()
    //         .to_string()
    // }

    /// Check to see if `status.bytes_uploaded` >= `status.size`
    pub fn upload_complete(&self) -> bool {
        self.status.bytes_uploaded >= self.status.size
    }

    /// Builds and returns the values to be added to the UPLOAD_METADATA value
    /// for this upload
    ///
    /// Calculates filesize and sets mimetype if present
    pub fn data(&self) -> Result<HashMap<String, String>, TusError> {
        let mut h = HashMap::new();
        h.insert(
            "filename".to_string(),
            self.file_path.to_str().unwrap().to_string(),
        );
        if let Some(mime) = &self.mime_type {
            h.insert("filetype".to_string(), mime.clone());
        }
        if let Some(extra) = &self.extra_meta {
            h.extend(extra.clone());
        }
        Ok(h)
    }

    /// Builds and returns the values to be added to the UPLOAD_METADATA value
    /// for this upload.
    ///
    /// - converts the key:value pairs to base64 encoding
    /// - returns all values as a string "key:value,key:value,..."
    ///
    /// Calculates filesize and sets mimetype if present
    pub fn data64(&self) -> Result<String, TusError> {
        let d = self
            .data()?
            .into_iter()
            .map(|(k, v)| {
                format!(
                    "{} {}",
                    k,
                    base64::engine::general_purpose::STANDARD.encode(v)
                )
            })
            .collect::<Vec<String>>()
            .join(",");
        Ok(d)
    }

    /// Convenience method to create a new meta data struct with updated `status` value
    pub fn with_bytes_uploaded(&self, bytes_uploaded: usize) -> Self {
        UploadMeta {
            status: UploadStatus {
                bytes_uploaded,
                ..self.status.clone()
            },
            ..self.clone()
        }
    }

    /// Convenience method to update remote_dest property
    pub fn with_remote_dest(&self, remote_url: String) -> Result<Self, TusError> {
        let remote_url = Url::parse(&remote_url)
            .map_err(|_| TusError::StringParseError("Malformed Url".to_string()))?;
        Ok(UploadMeta {
            remote_url: Some(remote_url),
            ..self.clone()
        })
    }
}
