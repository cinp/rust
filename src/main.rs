use std::error;
use std::fmt;
use std::iter::Map;

use http::header::{
  HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_CHARSET, CONTENT_TYPE, USER_AGENT,
};
use http::uri::Parts;
use http::uri::PathAndQuery;
use http::Method;
use http::Uri;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize)]
struct Login
{
  pub username: String,
  pub password: String,
}

#[derive(Serialize, Deserialize)]
struct FieldParamater
{
  pub name:            String,
  pub doc:             String,
  pub path:            String,
  #[serde(rename = "type")]
  pub type_:           String,
  pub length:          u16,
  pub uri:             String,
  pub allowed_schemes: Vec<String>,
  //choices        []interface{} `json:"choices"`
  pub is_array:        bool,
  //default        interface{}   `json:"default"`
  pub mode:            String,
  pub required:        bool,
}

#[derive(Serialize, Deserialize)]
struct Describe
{
  pub name:                String,
  pub doc:                 String,
  pub path:                String,
  // Namespace
  pub api_version:         String,
  pub multi_uri_max:       u16,
  pub namespaces:          Vec<String>,
  pub models:              Vec<String>,
  // Model
  pub constants:           Map<String, String>,
  pub fields:              Vec<FieldParamater>,
  pub actions:             Vec<String>,
  pub not_allowed_methods: Vec<String>,
  pub list_filters:        Map<String, Vec<FieldParamater>>,
  // Actions
  pub return_type:         FieldParamater,
  #[serde(rename = "static")]
  pub static_:             bool,
  pub paramaters:          Vec<FieldParamater>,
}

#[derive(Debug)]
pub enum Error
{
  InvalidURL,
  HTTP(hyper::Error),
  Parse(serde_json::Error),
}

impl fmt::Display for Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    match *self
    {
      Error::InvalidURL => write!(f, "invalid URL"),
      Error::HTTP(err) => err.fmt(f),
      Error::Parse(err) => err.fmt(f),
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
      Error::HTTP(ref err) => Some(err),
      Error::Parse(ref err) => Some(err),
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

struct CInP
{
  api_root:   Parts,
  client:     Client<HttpConnector, Body>,
  path_regex: Regex,
}

impl CInP
{
  fn new(api_root: &str) -> CInP
  {
    let uri = match api_root.parse::<Uri>()
    {
      Ok(res) => res,
      Err(err) => panic!("api_root is not valid URI: {}", err),
    };

    let scheme = uri.scheme_str().unwrap();
    if !(scheme == "http" || scheme == "https")
    {
      panic!("host does not start with http(s)://");
    }

    let path_regex = format!(
      "^({})(([a-zA-Z0-9\\-_.!~*]+/)*)([a-zA-Z0-9\\-_.!~*]+)?(:([a-zA-Z0-9\\-_.!~*\']*:)*)?(\\([a-zA-Z0-9\\-_.!~*]+\\))?$",
      uri.path()
    );

    CInP {
      api_root:   uri.into_parts(),
      client:     Client::new(),
      path_regex: Regex::new(&path_regex).unwrap(),
    }
  }

  fn create_request<'a>(
    &self,
    method: Method,
    uri: &'a str,
    data: Option<impl Serialize>,
    headers: Option<HeaderMap>,
  ) -> Result<Request<Vec<u8>>, Error>
  {
    if !self.path_regex.is_match(uri)
    {
      return Err(Error::InvalidURL);
    }

    let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(4);
    headers.insert(
      USER_AGENT,
      HeaderValue::from_bytes(b"Rust CInP client").unwrap(),
    );
    headers.insert(
      ACCEPT,
      HeaderValue::from_bytes(b"application/json").unwrap(),
    );
    headers.insert(ACCEPT_CHARSET, HeaderValue::from_bytes(b"utf-8").unwrap());
    headers.insert(
      CONTENT_TYPE,
      HeaderValue::from_bytes(b"application/json;charset=utf-8").unwrap(),
    );

    headers.insert(
      HeaderName::from_bytes(b"CInP-Version").unwrap(),
      HeaderValue::from_bytes(b"1.0").unwrap(),
    );

    let mut target_uri = Parts::default();
    target_uri.path_and_query = Some(PathAndQuery::try_from(uri).unwrap());
    target_uri.scheme = self.api_root.scheme.clone();
    target_uri.authority = self.api_root.authority.clone();
    let target_uri = Uri::from_parts(target_uri).unwrap();

    let body: Vec<u8> = match data
    {
      Some(value) => serde_json::to_vec(&value).unwrap().into(),
      None => Vec::new(),
    };

    println!("------ Request ------");
    println!("{:?}", body);

    let mut request = Request::builder()
      .method(method)
      .uri(target_uri)
      .body(body)
      .expect("request build");

    *request.headers_mut() = headers;

    return Ok(request);
  }

  /// ```
  /// pass the type as "Value" to get a Map back
  /// ```
  async fn request<'a, T>(
    &self,
    method: Method,
    uri: &'a str,
    data: Option<impl Serialize>,
    headers: Option<HeaderMap>,
  ) -> Result<(T, &HeaderMap), Error>
  where
    T: Deserialize<'a>,
  {
    let request = self.create_request(method, uri, data, headers)?;

    let res: Response<Vec<u8>> = match self.client.request(request).await
    {
      Ok(res) => res,
      Err(err) => panic!("{:?}", err),
      // Err(err) => match err.kind()
      // {
      //   io::ErrorKind::ConnectionRefused
      //   | io::ErrorKind::ConnectionAborted
      //   | io::ErrorKind::ConnectionReset => panic!("Connect Error: {:?}", err),
      //   other_error =>
      //   {
      //     panic!("other error: {:?}", other_error)
      //   }
      // },
    };

    let headers = res.headers();
    let body = res.into_body();

    let value: T = match serde_json::from_slice(&body)
    {
      Ok(res) => res,
      Err(e) => return Err(Error::Parse(e)),
    };

    Ok((value, headers))
  }

  async fn describe<'a>(&self, uri: &'a str) -> Result<(Describe, &str), Error>
  {
    let (data, headers): (Describe, &HeaderMap) = self
      .request(Method::from_bytes(b"DESCRIBE").unwrap(), uri, None, None)
      .await?;

    Ok((data, headers["type"].to_str().unwrap()))
  }
}

fn main()
{
  let runtime = Runtime::new().unwrap();

  let cinp = CInP::new("http://localhost:8000/api/v1/");

  let req = cinp.request(
    Method::from_bytes(b"DESCRIBE").unwrap(),
    "/api/v1/Site/Site",
    None,
    None,
  );

  let (data, req_type) = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  let req = cinp.request<Value>(
    Method::from_bytes(b"DESCRIBE").unwrap(),
    "/api/v1/Building/",
    None,
    None,
  );

  let data = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  let data = Login {
    username: String::from("root"),
    password: String::from("root"),
  };

  let req = cinp.request<Value>(
    Method::from_bytes(b"CALL").unwrap(),
    "/api/v1/Auth/User(login)",
    Some(data),
    None,
  );

  let data = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);
}
