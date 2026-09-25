use axum::{
	Json, Router,
	extract::{Path, State},
	http::StatusCode,
	routing::get,
};
use serde::Serialize;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{self, Problem};

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

async fn problems(State(pool): State<SqlitePool>) -> Result<Json<Vec<Problem>>, StatusCode> {
	let list = db::problems(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
	Ok(Json(list))
}

async fn problem(State(pool): State<SqlitePool>, Path(slug): Path<String>) -> Result<Json<Problem>, StatusCode> {
	let found = db::problem(&pool, &slug).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
	found.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub fn app(pool: SqlitePool) -> Router {
	Router::new()
		.route("/api/ping", get(ping))
		.route("/api/problems", get(problems))
		.route("/api/problems/{slug}", get(problem))
		.with_state(pool)
}
