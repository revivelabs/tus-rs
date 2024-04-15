use std::{io, num::ParseIntError};

use crate::tus;

/// Enumerates the errors which can occur during operation
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum TusError {
    /// UnexpectedStatusCode: ({0}) : {1}
    UnexpectedStatusCode(usize, String),

    /// The file specified was not found by the server.
    NotFoundError,

    /// Checksum mismatch error
    ChecksumMismatch,

    /// Invalid filename: {0}
    InvalidFilename(String),

    /// Empty filename
    EmptyFilename,

    /// Missing requred header: {0}
    MissingHeader(String),

    /// Missing upload URL - must create one with creation extension first
    MissingUploadUrl,

    /// Invalid Header: {0}
    InvalidHeader(String),

    /// Invalid Header Value: {0}
    InvalidHeaderValue(String),

    /// IO error: {0}
    IoError(io::Error),

    /// Int parsing error: {0}
    ParsingError(ParseIntError),

    /// String parsing error: {0}
    StringParseError(String),

    /// The size of the specified file, and the file size reported by the server do not match.
    UnequalSizeError,

    /// Unable to read the file specified: {0}.
    FileReadError(String),

    /// The `Client` tried to upload the file with an incorrect offset.
    WrongUploadOffsetError,

    /// The specified file is larger that what is supported by the server.
    FileTooLarge,

    /// An error occurred in the HTTP handler: {0}
    HttpHandlerError(tus::errors::TusAPIError),

    /// Request Error: {0}
    RequestError(String),

    /// Reqwest Error: {0}
    ReqwestError(reqwest::Error),

    /// Bad Request - {0}
    BadRequest(String),

    /// Serde serialize error
    SerdeError,

    /// Invalid to str
    ToStrError(reqwest::header::ToStrError),
}

impl From<reqwest::header::ToStrError> for TusError {
    fn from(value: reqwest::header::ToStrError) -> Self {
        TusError::ToStrError(value)
    }
}

impl From<io::Error> for TusError {
    fn from(e: io::Error) -> Self {
        TusError::IoError(e)
    }
}

impl From<ParseIntError> for TusError {
    fn from(e: ParseIntError) -> Self {
        TusError::ParsingError(e)
    }
}
