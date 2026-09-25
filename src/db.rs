use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::path::{Path, PathBuf};

pub const DATA: &str = "data";

#[derive(Serialize, sqlx::FromRow)]
pub struct Problem {
	#[serde(skip)]
	pub key: i64,
	pub id: String,
	pub name: String,
	pub title: String,
	pub time_limit: i64,
	pub memory_limit: i64,
	pub public: bool,
	pub created_at: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Contest {
	pub id: i64,
	pub slug: String,
	pub name: String,
	pub start_at: String,
	pub end_at: Option<String>,
	pub created_at: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ContestProblem {
	pub label: String,
	pub id: String,
	pub title: String,
}

pub async fn open() -> SqlitePool {
	std::fs::create_dir_all(Path::new(DATA).join("problems")).unwrap();
	let opts = SqliteConnectOptions::new()
		.filename(Path::new(DATA).join("xioioi.db"))
		.create_if_missing(true)
		.journal_mode(SqliteJournalMode::Wal)
		.foreign_keys(true);
	let pool = SqlitePool::connect_with(opts).await.unwrap();
	sqlx::migrate!().run(&pool).await.unwrap();
	pool
}

pub fn problem_zip(id: &str) -> PathBuf {
	Path::new(DATA).join("problems").join(format!("{id}.zip"))
}

pub async fn problems(pool: &SqlitePool) -> sqlx::Result<Vec<Problem>> {
	sqlx::query_as("SELECT * FROM problem_view WHERE public = 1 ORDER BY name")
		.fetch_all(pool)
		.await
}

pub async fn problem(pool: &SqlitePool, id: &str) -> sqlx::Result<Option<Problem>> {
	sqlx::query_as("SELECT * FROM problem_view WHERE id = ? AND public = 1")
		.bind(id)
		.fetch_optional(pool)
		.await
}

pub async fn contests(pool: &SqlitePool) -> sqlx::Result<Vec<Contest>> {
	sqlx::query_as("SELECT * FROM contests ORDER BY start_at DESC")
		.fetch_all(pool)
		.await
}

pub async fn contest(pool: &SqlitePool, slug: &str) -> sqlx::Result<Option<Contest>> {
	sqlx::query_as("SELECT * FROM contests WHERE slug = ?")
		.bind(slug)
		.fetch_optional(pool)
		.await
}

pub async fn contest_problems(pool: &SqlitePool, contest: i64) -> sqlx::Result<Vec<ContestProblem>> {
	sqlx::query_as("SELECT label, id, title FROM contest_problem_view WHERE contest_id = ? ORDER BY label")
		.bind(contest)
		.fetch_all(pool)
		.await
}

pub enum Order {
	Id,
	Name,
}

pub async fn search(pool: &SqlitePool, q: &str, order: Order) -> sqlx::Result<Vec<Problem>> {
	let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
	let pattern = format!("%{escaped}%");
	let sql = match order {
		Order::Id => "SELECT * FROM problem_view WHERE public = 1 AND (id LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\') ORDER BY id",
		Order::Name => "SELECT * FROM problem_view WHERE public = 1 AND (id LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\') ORDER BY title",
	};
	sqlx::query_as(sql).bind(pattern).fetch_all(pool).await
}
