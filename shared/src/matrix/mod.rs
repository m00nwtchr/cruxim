use crux_core::{macros::Effect, render::Render};
use crux_http::Http;
use crux_kv::KeyValue;
use log::info;
use serde::{Deserialize, Serialize};

pub mod homeserver;
pub use homeserver::HomeserverConfig;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
	HomeserverCfg(HomeserverConfig),

	Discover,

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
	homeserver: Option<Homeserver>
	user_name: String, 
	pub status: Status,
}

#[derive(Serialize, Deserialize, Clone)]
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
			Event::Discover => {
				
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
