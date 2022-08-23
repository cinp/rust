use std::collections::HashMap;

use http::Method;
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamaterType
{
  pub name:            String,
  pub path:            Option<String>,
  #[serde(rename = "type")]
  pub type_:           String,
  pub length:          Option<u16>,
  pub uri:             Option<String>,
  #[serde(rename = "allowed-schemes")]
  pub allowed_schemes: Option<Vec<String>>,
  //choices        []interface{} `json:"choices"`
  pub is_array:        Option<bool>,
  //default        interface{}   `json:"default"`
  pub mode:            Option<String>,
  pub required:        Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReturnType
{
  pub path:            Option<String>,
  #[serde(rename = "type")]
  pub type_:           String,
  pub length:          Option<u16>,
  pub uri:             Option<String>,
  #[serde(rename = "allowed-schemes")]
  pub allowed_schemes: Option<Vec<String>>,
  //choices        []interface{} `json:"choices"`
  pub is_array:        Option<bool>,
  //default        interface{}   `json:"default"`
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FieldType
{
  pub name:            String,
  pub path:            Option<String>,
  #[serde(rename = "type")]
  pub type_:           String,
  pub length:          Option<u16>,
  pub uri:             Option<String>,
  #[serde(rename = "allowed-schemes")]
  pub allowed_schemes: Option<Vec<String>>,
  //choices        []interface{} `json:"choices"`
  pub is_array:        Option<bool>,
  //default        interface{}   `json:"default"`
  pub mode:            Option<String>,
  pub required:        Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Describe
{
  pub name:                String,
  pub doc:                 Option<String>,
  pub path:                String,
  // Namespace
  #[serde(rename = "api-version")]
  pub api_version:         Option<String>,
  #[serde(rename = "multi-uri-max")]
  pub multi_uri_max:       Option<u16>,
  pub namespaces:          Option<Vec<String>>,
  pub models:              Option<Vec<String>>,
  // Model
  pub constants:           Option<HashMap<String, String>>,
  pub fields:              Option<Vec<FieldType>>,
  pub actions:             Option<Vec<String>>,
  #[serde(rename = "not-allowed-methods")]
  pub not_allowed_methods: Option<Vec<String>>,
  pub list_filters:        Option<HashMap<String, Vec<FieldType>>>,
  // Actions
  #[serde(rename = "return-type")]
  pub return_type:         Option<ReturnType>,
  #[serde(rename = "static")]
  pub static_:             Option<bool>,
  pub paramaters:          Option<Vec<FieldType>>,
}

impl Client
{
  pub async fn describe(&self, uri: &str) -> Result<(Describe, String), Error>
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
