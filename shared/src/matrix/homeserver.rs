use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HomeserverConfig {
	ServerName(String),
	ServerUrl(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Homeserver {}

pub fn crux_response_to_http<T: Default>(mut r: crux_http::Response<T>) -> http::Response<T> {
	let mut resp = http::Response::default();

	if let Some(body) = r.take_body() {
		*resp.body_mut() = body;
	}

	resp
}
