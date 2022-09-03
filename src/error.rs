use std::collections::HashMap;
use std::error;
use std::fmt;

#[derive(Debug)]
pub struct InvalidRequest
{
  exception: Option<String>,
  error:     Option<String>,
  message:   Option<String>,
  data:      HashMap<String, String>,
}

impl InvalidRequest
{
  pub fn new(data: HashMap<String, String>) -> InvalidRequest
  {
    let exception = data.get(&"exception".to_string());
    let error = data.get(&"error".to_string());
    let message = data.get(&"message".to_string());

    let mut data = data.clone();

    data.remove(&"exception".to_string());
    data.remove(&"error".to_string());
    data.remove(&"message".to_string());

    InvalidRequest {
      exception: exception.map(|s| s.to_string()),
      error:     error.map(|s| s.to_string()),
      message:   message.map(|s| s.to_string()),
      data:      data,
    }
  }
}

impl fmt::Display for InvalidRequest
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    let blank = "".to_string();
    let message = self.message.as_ref().unwrap_or(&blank);
    let exception = self.exception.as_ref().unwrap_or(&blank);
    let error = self.error.as_ref().unwrap_or(&blank);
    write!(
      f,
      "message: '{}', exception: '{}', error: '{}'",
      message, exception, error,
    )
  }
}

impl error::Error for InvalidRequest
{
  fn description(&self) -> &str
  {
    &self.message.as_ref().unwrap()
  }
}

#[derive(Debug)]
pub struct ProtocolError
{
  pub detail: HashMap<String, String>,
}

impl fmt::Display for ProtocolError
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    //self.detail.fmt(f)
    write!(f, "Protocol Error")
  }
}

impl error::Error for ProtocolError
{
  fn description(&self) -> &str
  {
    //tmp = format!("{:?}", self.detail).as_str()
    "Protocol Error"
  }
}

#[derive(Debug)]
pub enum Error
{
  InvalidURL,
  ResponseToLarge,
  HTTP(hyper::Error),
  Parse(serde_json::Error),
  ServerError(ProtocolError),
  InvalidRequest(InvalidRequest),
  InvalidSession,
  NotAuthorized,
  NotFound,
}

impl fmt::Display for Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    match *self
    {
      Error::InvalidURL => write!(f, "Invalid URL"),
      Error::ResponseToLarge => write!(f, "Response to Large"),
      Error::HTTP(ref err) => err.fmt(f),
      Error::Parse(ref err) => err.fmt(f),
      Error::ServerError(ref err) => err.fmt(f),
      Error::InvalidRequest(ref err) => err.fmt(f),
      Error::InvalidSession => write!(f, "Invalid Session"),
      Error::NotAuthorized => write!(f, "Not Authorized"),
      Error::NotFound => write!(f, "Not Found"),
    }
  }
}

impl error::Error for Error
{
  fn source(&self) -> Option<&(dyn error::Error + 'static)>
  {
    match &*self
    {
      Error::InvalidURL => None,
      Error::ResponseToLarge => None,
      Error::HTTP(ref err) => Some(err),
      Error::Parse(ref err) => Some(err),
      Error::ServerError(ref err) => Some(err),
      Error::InvalidRequest(ref err) => Some(err),
      Error::InvalidSession => None,
      Error::NotAuthorized => None,
      Error::NotFound => None,
    }
  }
}

impl From<hyper::Error> for Error
{
  fn from(err: hyper::Error) -> Error
  {
    Error::HTTP(err)
  }
}

impl From<serde_json::Error> for Error
{
  fn from(err: serde_json::Error) -> Error
  {
    Error::Parse(err)
  }
}

impl From<InvalidRequest> for Error
{
  fn from(err: InvalidRequest) -> Error
  {
    Error::InvalidRequest(err)
  }
}
