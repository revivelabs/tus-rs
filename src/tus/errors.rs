/// The errors a TusAPI can return
#[derive(thiserror::Error, displaydoc::Display, Debug)]
pub enum TusAPIError {
    /// Underlying Error
    UnderlyingError,

    /// Could not fetch TUS status
    CouldNotFetchStatus,

    /// Could not fetch server info
    CouldNotFetchServerInfo,

    /// Could not retrieve offset
    CouldNotRetrieveOffset,

    /// Could not retriev location
    CouldNotRetrieveLocation,

    /// Request failed
    FailedRequest,
}
