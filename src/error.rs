use std::error;
use std::fmt;

#[derive(Debug)]
pub struct ProtocolError
{
  pub msg: String,
}

impl fmt::Display for ProtocolError
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    self.msg.fmt(f)
  }
}

impl error::Error for ProtocolError
{
  fn description(&self) -> &str
  {
    &self.msg
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
  BadRequest(ProtocolError),
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
      Error::BadRequest(ref err) => err.fmt(f),
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
    match *self
    {
      Error::InvalidURL => None,
      Error::ResponseToLarge => None,
      Error::HTTP(ref err) => Some(err),
      Error::Parse(ref err) => Some(err),
      Error::ServerError(ref err) => Some(err),
      Error::BadRequest(ref err) => Some(err),
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
