use crux_core::{macros::Effect, render::Render};
use crux_http::{http_types::StatusCode, Http};
use crux_kv::KeyValue;
use log::{debug, info};
use ruma::{
	api::{
		client::{discovery, discovery::discover_homeserver},
		IncomingResponse,
	},
	exports::http::response,
};
use serde::{Deserialize, Serialize};

pub mod homeserver;
pub use homeserver::HomeserverConfig;

use crate::matrix::homeserver::{crux_response_to_http, Homeserver};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
	HomeserverCfg(HomeserverConfig),

	Discover,

	#[serde(skip)]
	DiscoverResponse(crux_http::Result<crux_http::Response<Vec<u8>>>),
	#[serde(skip)]
	ValidateHomeserverUrl(String),
	#[serde(skip)]
	Error(String),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub enum Status {
	#[default]
	None,
	Info(String),
	Error(String),
}

#[derive(Default, Debug)]
pub struct Model {
	homeserver_cfg: Option<HomeserverConfig>,
	homeserver: Option<Homeserver>,
	user_name: String,
	pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewModel {
	pub status: Status,
}

#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(Effect)]
pub struct Capabilities {
	pub http: Http<Event>,
	pub store: KeyValue<Event>,
	// pub passkey: Passkey<Event>,
	pub render: Render<Event>,
}

#[derive(Default)]
pub struct MatrixCore;

impl crux_core::App for MatrixCore {
	type Event = Event;
	type Model = Model;
	type ViewModel = ViewModel;
	type Capabilities = Capabilities;

	fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
		info!("{event:?}");

		let homeserver_cfg = model
			.homeserver_cfg
			.as_ref()
			.cloned()
			.unwrap_or(HomeserverConfig::ServerName("matrix.org".to_owned()));

		match event {
			Event::HomeserverCfg(cfg) => model.homeserver_cfg = Some(cfg),
			Event::Discover => match homeserver_cfg {
				HomeserverConfig::ServerName(name) => caps
					.http
					.get(format!("https://{name}/.well-known/matrix/client"))
					.send(Event::DiscoverResponse),
				HomeserverConfig::ServerUrl(url) => {
					self.update(Event::ValidateHomeserverUrl(url), model, caps);
				}
			},
			Event::DiscoverResponse(res) => match res {
				Ok(res) => {
					let res: http::Response<Vec<u8>> = res.try_into().expect("invalid response");
					let res = discover_homeserver::Response::try_from_http_response(res);

					if let Ok(res) = res {
						self.update(
							Event::ValidateHomeserverUrl(res.homeserver.base_url),
							model,
							caps,
						);
					}
				}
				Err(e) => {}
			},
			Event::ValidateHomeserverUrl(url) => {
				caps.http
					.get(format!("https://{url}/_matrix/client/versions"))
					.send(|res| {
						if let Ok(res) = res {
							if res.status() == StatusCode::Ok {
								let res: http::Response<Vec<u8>> =
									res.try_into().expect("invalid response");
								let resp = discovery::get_supported_versions::Response::try_from_http_response(res);

								if resp.is_ok() {
									return Event::Error("Homeserver url valid".to_string());
									// return Event::HomeserverCfg(HomeserverConfig::ServerUrl(url));
								}
							}
						}

						Event::Error("Invalid homeserver url".to_string())
					});
			}
			Event::Error(e) => {
				model.status = Status::Error(e);
				caps.render.render();
			}
		}
	}

	fn view(&self, model: &Self::Model) -> Self::ViewModel {
		ViewModel {
			status: model.status.clone(),
		}
	}
}

#[cfg(test)]
mod tests {
	use crux_core::{assert_effect, testing::AppTester};
	use crux_http::testing::ResponseBuilder;

	use crate::matrix::{Effect, Event, HomeserverConfig, MatrixCore, Model};

	#[test]
	pub fn discover() {
		env_logger::init();
		let app = AppTester::<MatrixCore, _>::default();

		let homeserver_name = "m00nlit.dev";
		let mut model = Model {
			homeserver_cfg: Some(HomeserverConfig::ServerName(homeserver_name.to_owned())),
			..Default::default()
		};

		let event = Event::Discover;

		let update = app.update(event, &mut model);
		assert_effect!(update, Effect::Http(_));

		let update = app.update(
			Event::DiscoverResponse(Ok(ResponseBuilder::ok()
				.body(
					r#"{"m.homeserver": {"base_url": "https://matrix.m00nlit.dev"}}"#
						.as_bytes()
						.to_vec(),
				)
				.build())),
			&mut model,
		);
	}
}
