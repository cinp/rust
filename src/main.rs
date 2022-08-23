use std::collections::HashMap;

use cinp::client::Client;
//use cinp::describe::Describe;
//use http::header::HeaderMap;
use http::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::runtime::Runtime;

#[derive(Serialize)]
struct LoginRequest
{
  pub username: String,
  pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Structure
{
  #[serde(skip_serializing_if = "Option::is_none")]
  pub blueprint:     Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub foundation:    Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname:      Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub site:          Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub config_uuid:   Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub config_values: Option<HashMap<String, String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created:       Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated:       Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state:         Option<String>,
}

fn main()
{
  let runtime = Runtime::new().unwrap();

  let mut cinp = Client::new("http://localhost:8888/api/v1/");

  let data = LoginRequest {
    username: "root".to_string(),
    password: "root".to_string(),
  };

  println!("------");
  let req = cinp.request::<String>(
    Method::from_bytes(b"CALL").unwrap(),
    "/api/v1/Auth/User(login)",
    Some(data),
    None,
  );

  let (data, _) = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  cinp.add_header(b"AUTH_ID", b"root");
  cinp.add_header(b"AUTH_TOKEN", data.as_bytes());

  println!("------");
  let req = cinp.request::<Value>(
    Method::from_bytes(b"GET").unwrap(),
    "/api/v1/Building/Structure:1:",
    None::<()>,
    None,
  );

  let data = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  println!("------");
  let req = cinp.list("/api/v1/Building/Structure");

  let (data, _, _, _) = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  println!("------");
  let req = cinp.get::<Value>("/api/v1/Building/Structure:1:");

  let data = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  println!("------");
  let data = Structure {
    blueprint: Some("/api/v1/BluePrint/StructureBluePrint:manual-structure-base:".to_string()),
    foundation: Some("/api/v1/Building/Foundation:command-01:".to_string()),
    hostname: Some("teststuff".to_string()),
    site: Some("/api/v1/Site/Site:demo:".to_string()),
    ..Default::default()
  };
  let req = cinp.create::<Structure>("/api/v1/Building/Structure", data);

  let data = runtime.block_on(req).expect("got an error");
  println!("{:?}", data);

  // println!("------");
  // let req = cinp.describe("/api/v1/Building/Foundation");
  //
  // let (data, req_type) = runtime.block_on(req).expect("got an error");
  // println!("{:?}", req_type);
  // println!("{:?}", data);
}
