//! Utilities for resolving a devices on the LAN.
//!
//! Examples
//!
//! ```rust,no_run
//! use mdns::Error;
//! use std::time::Duration;
//!
//! const SERVICE_NAME: &'static str = "_googlecast._tcp.local";
//! const HOST: &'static str = "mycast._googlecast._tcp.local";
//!
//! #[async_std::main]
//! async fn main() -> Result<(), Error> {
//!     if let Some(response) = mdns::resolve::one(SERVICE_NAME, HOST, Duration::from_secs(15)).await? {
//!         println!("{:?}", response);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::{Error, Response};
use futures_util::{pin_mut, StreamExt, TryFutureExt};
use std::time::Duration;

/// Resolve a single device by hostname
pub async fn one<S>(
    service_names: Vec<String>,
    host_name: S,
    timeout: Duration,
) -> Result<Option<Response>, Error>
where
    S: AsRef<str>,
{
    // by setting the query interval higher than the timeout we ensure we only make one query
    let stream = crate::discover::all(service_names, timeout * 2)?.listen();
    pin_mut!(stream);

    let process = async {
        while let Some(Ok(response)) = stream.next().await {
            match response.hostname() {
                Some(found_host) if found_host == host_name.as_ref() => return Some(response),
                _ => {}
            }
        }

        None
    };

    async_std::future::timeout(timeout, process)
        .map_err(|e| e.into())
        .await
}

/// Resolve multiple devices by hostname
pub async fn multiple<S>(
    service_names: Vec<String>,
    host_names: &[S],
    timeout: Duration,
) -> Result<Vec<Response>, Error>
where
    S: AsRef<str>,
{
    // by setting the query interval higher than the timeout we ensure we only make one query
    let stream = crate::discover::all(service_names, timeout * 2)?.listen();
    pin_mut!(stream);

    let mut found = Vec::new();

    let process = async {
        while let Some(Ok(response)) = stream.next().await {
            match response.hostname() {
                Some(found_host) if host_names.iter().any(|s| s.as_ref() == found_host) => {
                    found.push(response);

                    if found.len() == host_names.len() {
                        return;
                    }
                }
                _ => {}
            }
        }
    };

    match async_std::future::timeout(timeout, process).await {
        Ok(()) => Ok(found),
        Err(e) => Err(e.into()),
    }
}
