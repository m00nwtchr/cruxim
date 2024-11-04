use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HomeserverConfig {
	ServerName(String),
	ServerUrl(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Homeserver {}
