use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct Pong {
	pong: bool,
	t: u128,
}

async fn ping() -> Json<Pong> {
	let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
	let pong = Pong {
		pong: true,
		t: now.as_millis(),
	};
	Json(pong)
}

pub fn app() -> Router {
	Router::new().route("/api/ping", get(ping))
}
