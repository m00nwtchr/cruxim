use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use crossbeam_channel::{unbounded, Sender};
use crux_http::{
	protocol::{HttpHeader, HttpRequest, HttpResponse},
	HttpError, Result,
};
use log::{debug, info};
use reqwest::{Client, Method};
use shared::matrix::{Effect, Event, HomeserverConfig, MatrixCore};
use uniffi::deps::anyhow;

pub async fn http_request(
	HttpRequest {
		method,
		url,
		headers,
		body,
	}: &HttpRequest,
) -> Result<HttpResponse> {
	let client = Client::new();
	let method =
		Method::from_bytes(method.as_bytes()).map_err(|e| HttpError::Url(e.to_string()))?;

	let headers = headers.iter().map(|header| {
		let name = reqwest::header::HeaderName::from_bytes(header.name.as_bytes())
			.expect("Invalid header name");
		let value = reqwest::header::HeaderValue::from_bytes(header.value.as_bytes())
			.expect("Invalid header value");

		(name, value)
	});

	let request = client
		.request(method, url)
		.headers(reqwest::header::HeaderMap::from_iter(headers))
		.body(body.to_vec())
		.build()
		.map_err(|e| HttpError::Url(e.to_string()))?;

	let response = client
		.execute(request)
		.await
		.map_err(|e| HttpError::Io(e.to_string()))?;

	let headers = response
		.headers()
		.iter()
		.map(|(name, value)| {
			value
				.to_str()
				.map(|v| HttpHeader {
					name: name.to_string(),
					value: v.to_string(),
				})
				.map_err(|e| HttpError::Io(e.to_string()))
		})
		.collect::<Result<Vec<HttpHeader>>>()?;

	Ok(HttpResponse {
		status: response.status().as_u16(),
		headers,
		body: response
			.bytes()
			.await
			.map_err(|e| HttpError::Io(e.to_string()))?
			.to_vec(),
	})
}

pub type Core = Arc<shared::Core<Effect, MatrixCore>>;

pub fn new() -> Core {
	Arc::new(shared::Core::new())
}

pub fn update(core: &Core, event: Event, tx: &Arc<Sender<Effect>>) -> anyhow::Result<()> {
	debug!("event: {:?}", event);

	for effect in core.process_event(event) {
		process_effect(core, effect, tx)?;
	}
	Ok(())
}

pub fn process_effect(core: &Core, effect: Effect, tx: &Arc<Sender<Effect>>) -> anyhow::Result<()> {
	debug!("effect: {:?}", effect);

	match effect {
		render @ Effect::Render(_) => {
			tx.send(render).map_err(|e| anyhow!("{:?}", e))?;
		}

		Effect::Http(mut request) => {
			let core = core.clone();
			let tx = tx.clone();

			tokio::spawn(async move {
				info!("http");
				let response = http_request(&request.operation).await;

				for effect in core.resolve(&mut request, response.into()) {
					info!("thread effect: {:?}", effect);
					process_effect(&core, effect, &tx)?;
				}
				anyhow::Result::<()>::Ok(())
			});
		}

		kv @ Effect::KeyValue(_) => {
			tx.send(kv).map_err(|e| anyhow!("{:?}", e))?;
		}
	}
	Ok(())
}

#[tokio::test]
pub async fn main() -> anyhow::Result<()> {
	env_logger::init();

	let core = new();
	let event = Event::HomeserverCfg(HomeserverConfig::ServerName("m00nlit.dev".to_string()));
	let (tx, rx) = unbounded::<Effect>();

	update(&core, Event::Discover, &Arc::new(tx))?;
	while let Ok(effect) = rx.recv() {
		if let Effect::Render(_) = effect {
			let view = core.view();

			println!("{view:?}");
		}
	}

	// let (tx, rx) = unbounded::<Effect>();
	// update(&core, Event::Discover, &Arc::new(tx))?;
	// sleep(Duration::from_secs(5));
	// while let Ok(effect) = rx.recv() {
	// 	// println!("{effect:?}");
	// 	if let Effect::Render(_) = effect {
	// 		let view = core.view();
	//
	// 		println!("{view:?}");
	// 	}
	// }

	Ok(())
}
