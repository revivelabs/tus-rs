use crate::{error::TusError, tus::headers::Headers};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Enumerates the HTTP methods used by `tus::Client`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TusHttpMethod {
    /// Used to determine the offset at which the upload should be continued
    Head,

    Post,
    Get,

    /// Used to resume upload
    Patch,

    /// Used to get server info
    Options,

    /// used in Terminate extension
    Delete,
}

impl TusHttpMethod {
    pub fn to_method(&self) -> Method {
        match self {
            Self::Head => Method::HEAD,
            Self::Post => Method::POST,
            Self::Get => Method::GET,
            Self::Patch => Method::PATCH,
            Self::Options => Method::OPTIONS,
            Self::Delete => Method::DELETE,
        }
    }
}

impl fmt::Display for TusHttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents an HTTP request to be executed by the handler.
#[derive(Debug)]
pub struct HttpRequest<'a> {
    pub method: TusHttpMethod,
    pub headers: Headers,
    pub url: String,
    pub body: Option<&'a [u8]>,
}

/// Represents an HTTP response from the server.
#[derive(Debug)]
pub struct HttpResponse {
    pub headers: Headers,
    pub status_code: usize,
}

/// The required trait used by `tus::Client` to represent a handler to execute `HttpRequest`s.
pub trait HttpHandler {
    fn handle_request(&self, req: HttpRequest) -> Result<HttpResponse, TusError>;
}
