use http::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::client::Client;
use crate::error::Error;

impl Client
{
  pub async fn list(&self, uri: &str) -> Result<(Vec<String>, u64, u64, u64), Error>
  {
    let (data, _headers) = self
      .request::<Vec<String>>(Method::from_bytes(b"LIST").unwrap(), &uri, None::<()>, None)
      .await?;

    Ok((data, 0, 0, 0))
  }

  pub async fn get<T>(&self, uri: &str) -> Result<T, Error>
  where
    T: DeserializeOwned,
  {
    let (data, _) = self
      .request::<T>(Method::from_bytes(b"GET").unwrap(), &uri, None::<()>, None)
      .await?;

    Ok(data)
  }

  pub async fn create<T>(&self, uri: &str, data: T) -> Result<T, Error>
  where
    T: DeserializeOwned + Serialize,
  {
    let (data, _) = self
      .request::<T>(
        Method::from_bytes(b"CREATE").unwrap(),
        &uri,
        Some(data),
        None,
      )
      .await?;

    Ok(data)
  }
}
