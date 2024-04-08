use crate::{
    error::TusError,
    tus::{self, headers::Headers},
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

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
    Options,
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

    pub fn required_headers(&self) -> Vec<String> {
        let mut v = vec![tus::headers::TUS_RESUMABLE];

        match self {
            Self::Post => v.extend(vec![tus::headers::TUS_RESUMABLE]),
            Self::Get => v.extend(vec![tus::headers::TUS_RESUMABLE]),
            Self::Patch => v.extend(vec![
                tus::headers::CONTENT_TYPE,
                tus::headers::UPLOAD_OFFSET,
                tus::headers::UPLOAD_LENGTH,
            ]),
            Self::Delete => v.extend(vec![tus::headers::TUS_RESUMABLE]),
            _ => {}
        };

        v.into_iter().map(|s| s.to_string()).collect()
    }

    pub fn default_request_headers(&self) -> HashMap<String, String> {
        let mut all = HashMap::<String, String>::new();
        all.insert(
            String::from(tus::headers::TUS_RESUMABLE),
            String::from("1.0.0"),
        );

        all.extend(match self {
            Self::Head => HashMap::new(),
            // Self::Post => Method::POST,
            // Self::Get => Method::GET,
            // Self::Patch => Method::PATCH,
            // Self::Options => Method::OPTIONS,
            // Self::Delete => Method::DELETE,
            _ => HashMap::new(),
        });

        all
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
