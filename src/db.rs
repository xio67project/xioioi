use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::path::{Path, PathBuf};

pub const DATA: &str = "data";

#[derive(Serialize, sqlx::FromRow)]
pub struct Problem {
	pub id: i64,
	pub slug: String,
	pub title: String,
	pub time_limit: i64,
	pub memory_limit: i64,
	pub created_at: String,
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

pub fn problem_dir(slug: &str) -> PathBuf {
	Path::new(DATA).join("problems").join(slug)
}

pub async fn problems(pool: &SqlitePool) -> sqlx::Result<Vec<Problem>> {
	sqlx::query_as("SELECT * FROM problems ORDER BY id")
		.fetch_all(pool)
		.await
}

pub async fn problem(pool: &SqlitePool, slug: &str) -> sqlx::Result<Option<Problem>> {
	sqlx::query_as("SELECT * FROM problems WHERE slug = ?")
		.bind(slug)
		.fetch_optional(pool)
		.await
}
