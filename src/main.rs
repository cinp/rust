use std::error;
use std::fmt;

//use std::iter::Map;
use http::header::{
  HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_CHARSET, CONTENT_TYPE, USER_AGENT,
};
use http::uri::Parts;
use http::uri::PathAndQuery;
use http::Method;
use http::Uri;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::runtime::Runtime;

const MAX_ALLOWED_RESPONSE_SIZE: u64 = 40960;

#[derive(Serialize, Deserialize)]
struct Login
{
  pub username: String,
  pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FieldParamater
{
  pub name:     String,
  pub doc:      String,
  pub path:     String,
  #[serde(rename = "type")]
  pub type_:    String,
  pub length:   u16,
  pub uri:      String,
  //pub allowed_schemes: Vec<String>,
  //choices        []interface{} `json:"choices"`
  pub is_array: bool,
  //default        interface{}   `json:"default"`
  pub mode:     String,
  pub required: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Describe
{
  pub name:          String,
  pub doc:           String,
  pub path:          String,
  // Namespace
  pub api_version:   String,
  pub multi_uri_max: u16,
  //pub namespaces:          &'a Vec<String>,
  //pub models:              &'a Vec<String>,
  // Model
  //pub constants:           &'a Map<String, String>,
  //pub fields:              &'a Vec<FieldParamater>,
  //pub actions:             &'a Vec<String>,
  //pub not_allowed_methods: &'a Vec<String>,
  //pub list_filters:        &'a Map<String, Vec<FieldParamater>>,
  // Actions
  pub return_type:   FieldParamater,
  #[serde(rename = "static")]
  pub static_:       bool,
  //pub paramaters:    &'a Vec<FieldParamater>,
}

#[derive(Debug)]
pub enum Error
{
  InvalidURL,
  ResponseToLarge,
  HTTP(hyper::Error),
  Parse(serde_json::Error),
}

impl fmt::Display for Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    match *self
    {
      Error::InvalidURL => write!(f, "Invalid URL"),
      Error::ResponseToLarge => write!(f, "Response to Large"),
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
      Error::ResponseToLarge => None,
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
    extra_headers: Option<HeaderMap>,
  ) -> Result<Request<Body>, Error>
  {
    if !self.path_regex.is_match(uri)
    {
      return Err(Error::InvalidURL);
    }

    let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(match extra_headers
    {
      Some(val) => val.len() + 5,
      None => 5,
    });
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

    if extra_headers != None
    {
      for (key, value) in extra_headers.unwrap().iter()
      {
        headers.insert(key, value);
      }
    }

    let mut target_uri = Parts::default();
    target_uri.path_and_query = Some(PathAndQuery::try_from(uri).unwrap());
    target_uri.scheme = self.api_root.scheme.clone();
    target_uri.authority = self.api_root.authority.clone();
    let target_uri = Uri::from_parts(target_uri).unwrap();

    let body: Body = match data
    {
      Some(value) => serde_json::to_vec(&value).unwrap().into(),
      None => Body::empty(),
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
  async fn request<T>(
    &self,
    method: Method,
    uri: &str,
    data: Option<impl Serialize>,
    headers: Option<HeaderMap>,
  ) -> Result<(T, HeaderMap), Error>
  where
    T: DeserializeOwned,
  {
    let request = self.create_request(method, uri, data, headers)?;

    let res: Response<Body> = match self.client.request(request).await
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

    let response_content_length = match res.body().size_hint().upper()
    {
      Some(v) => v,
      None => MAX_ALLOWED_RESPONSE_SIZE + 1, // Just to protect ourselves from a malicious response
    };

    if response_content_length > MAX_ALLOWED_RESPONSE_SIZE
    {
      return Err(Error::ResponseToLarge);
    }

    let headers = res.headers().clone();

    let bytes = hyper::body::to_bytes(res.into_body()).await?;
    let value: T = serde_json::from_slice(&bytes).map_err(Error::Parse)?;

    Ok((value, headers))
  }

  async fn describe(&self, uri: &str) -> Result<(Describe, String), Error>
  {
    let (data, headers) = self
      .request::<Describe>(
        Method::from_bytes(b"DESCRIBE").unwrap(),
        &uri,
        None::<()>,
        None,
      )
      .await?;

    let target_type = headers.get("type").unwrap().to_str().unwrap().to_string();
    Ok((data, target_type))
  }
}

fn main()
{
  let runtime = Runtime::new().unwrap();

  let cinp = CInP::new("http://localhost:8000/api/v1/");

  let req = cinp.request(
    Method::from_bytes(b"DESCRIBE").unwrap(),
    "/api/v1/Site/Site",
    None::<()>,
    None,
  );

  let (data, req_type): (Describe, HeaderMap) = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  let req = cinp.request::<Describe>(
    Method::from_bytes(b"DESCRIBE").unwrap(),
    "/api/v1/Building/",
    None::<()>,
    None,
  );

  let data = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  let data = Login {
    username: String::from("root"),
    password: String::from("root"),
  };

  let req = cinp.request::<Value>(
    Method::from_bytes(b"CALL").unwrap(),
    "/api/v1/Auth/User(login)",
    Some(data),
    None,
  );

  let data = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);
}
