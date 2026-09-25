use axum::{
	Json, Router,
	extract::{Path, Query, State},
	http::StatusCode,
	routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "auth.rs"]
pub mod auth;
#[path = "db.rs"]
pub mod db;
#[path = "pkg.rs"]
pub mod pkg;

use db::{Contest, ContestProblem, Order, Problem};

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

#[derive(Deserialize)]
struct SearchQuery {
	#[serde(default)]
	q: String,
	order_by: Option<String>,
}

async fn problems(State(pool): State<SqlitePool>, Query(query): Query<SearchQuery>) -> Result<Json<Vec<Problem>>, StatusCode> {
	let order = match query.order_by.as_deref() {
		Some("name") => Order::Name,
		_ => Order::Id,
	};
	let list = db::search(&pool, query.q.trim(), order)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
	Ok(Json(list))
}

async fn problem(State(pool): State<SqlitePool>, Path(id): Path<String>) -> Result<Json<Problem>, StatusCode> {
	let found = db::problem(&pool, &id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
	found.map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn contests(State(pool): State<SqlitePool>) -> Result<Json<Vec<Contest>>, StatusCode> {
	let list = db::contests(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
	Ok(Json(list))
}

#[derive(Serialize)]
struct ContestFull {
	#[serde(flatten)]
	contest: Contest,
	problems: Vec<ContestProblem>,
}

async fn contest(State(pool): State<SqlitePool>, Path(slug): Path<String>) -> Result<Json<ContestFull>, StatusCode> {
	let err = |_| StatusCode::INTERNAL_SERVER_ERROR;
	let contest = db::contest(&pool, &slug).await.map_err(err)?.ok_or(StatusCode::NOT_FOUND)?;
	let problems = db::contest_problems(&pool, contest.id).await.map_err(err)?;
	Ok(Json(ContestFull { contest, problems }))
}

pub fn app(pool: SqlitePool) -> Router {
	Router::new()
		.route("/api/ping", get(ping))
		.route("/api/problems", get(problems))
		.route("/api/problems/{id}", get(problem))
		.route("/api/contest_list", get(contests))
		.route("/api/contests/{slug}", get(contest))
		.with_state(pool)
}
