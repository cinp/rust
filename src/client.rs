use std::collections::HashMap;

use http::header::{
  HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_CHARSET, CONTENT_TYPE, USER_AGENT,
};
use http::uri::Parts;
use http::uri::PathAndQuery;
use http::Method;
use http::Uri;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::{Body, Client as HTTPClient, Request, Response};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{Error, InvalidRequest};

const MAX_ALLOWED_RESPONSE_SIZE: u64 = 40960;

pub struct Client
{
  api_root:   Parts,
  client:     HTTPClient<HttpConnector, Body>,
  path_regex: Regex,
  headers:    HeaderMap,
}

impl Client
{
  pub fn new(api_root: &str) -> Client
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

    Client {
      api_root:   uri.into_parts(),
      client:     HTTPClient::new(),
      path_regex: Regex::new(&path_regex).unwrap(),
      headers:    HeaderMap::new(),
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
      Some(ref val) => val.len() + self.headers.len() + 5,
      None => self.headers.len() + 5,
    });

    if extra_headers.is_some()
    {
      for (key, value) in extra_headers.unwrap().iter()
      {
        headers.insert(key, value.clone());
      }
    }

    for (key, value) in self.headers.iter()
    {
      headers.insert(key, value.clone());
    }

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

    let body: Body = match data
    {
      Some(value) => serde_json::to_vec(&value).unwrap().into(),
      None => Body::empty(),
    };

    let mut request = Request::builder()
      .method(method)
      .uri(target_uri)
      .body(body)
      .expect("request build");

    *request.headers_mut() = headers;

    return Ok(request);
  }

  pub fn add_header(&mut self, name: &[u8], value: &[u8])
  {
    self.headers.insert(
      HeaderName::from_bytes(name).unwrap(),
      HeaderValue::from_bytes(value).unwrap(),
    );
  }

  /// ```
  /// pass the type as "Value" to get a Map back
  /// ```
  pub async fn request<T>(
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

    // if http_code not in ( 200, 201, 202, 400, 401, 403, 404, 500 ):
    //   raise ResponseError( 'HTTP code "{0}" unhandled'.format( resp.code ) )
    //
    // logging.debug( 'cinp: got HTTP code "{0}"'.format( http_code ) )

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
    let http_code = res.status().clone();
    let bytes = hyper::body::to_bytes(res.into_body()).await?;

    if http_code == 500 || http_code == 400
    {
      let data: HashMap<String, String> = match serde_json::from_slice(&bytes)
      {
        Ok(res) => res,
        Err(err) =>
        {
          let mut val = HashMap::new();
          val.insert(
            "message".to_string(),
            format!("Error Parsing Response: {}", err.to_string()),
          );
          val.insert(
            "response".to_string(),
            String::from_utf8(bytes.to_vec()).unwrap(),
          );
          val
        }
      };

      if http_code == 400
      {
        return Err(Error::InvalidRequest(InvalidRequest::new(data)));
      }
      else
      {
        //return Err(Error::ServerError(ProtocolError { detail: data }));
      }
    }

    if http_code == 401
    {
      return Err(Error::InvalidSession);
    }
    if http_code == 403
    {
      return Err(Error::NotAuthorized);
    }
    if http_code == 404
    {
      return Err(Error::NotFound);
    }

    let value: T = serde_json::from_slice(&bytes).map_err(Error::Parse)?;

    Ok((value, headers))
  }
}
