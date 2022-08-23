use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error
{
  InvalidURL,
  ResponseToLarge,
  HTTP(hyper::Error),
  Parse(serde_json::Error),
  ServerError(String),
  BadRequest(String),
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
      Error::ServerError(ref msg) => msg.fmt(f),
      Error::BadRequest(ref msg) => msg.fmt(f),
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
      Error::ServerError(ref msg) => Some(&<dyn error::Error + 'static>::from(msg)),
      Error::BadRequest(ref msg) => Some(<dyn error::Error>::from(msg)),
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
