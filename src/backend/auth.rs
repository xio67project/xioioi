use axum::extract::FromRequestParts;
use axum::http::{header, request::Parts};
use scrypt::Scrypt;
use scrypt::password_hash::PasswordVerifier;
use scrypt::phc::PasswordHash;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::convert::Infallible;

use super::db::{self, User};

pub const COOKIE: &str = "session";
pub const MAX_AGE: i64 = 30 * 24 * 60 * 60;

const DUMMY: &str = "$scrypt$ln=15,r=8,p=1$PkRCLhTptyR4I3T4eKN/Og$vktc3prdhEkcE62dBl4Jky7BB5j4E8qTbl+P+25NI5g";

pub struct Me(pub Option<User>);

pub fn cookie<'a>(parts: &'a Parts, name: &str) -> Option<&'a str> {
	let raw = parts.headers.get(header::COOKIE)?.to_str().ok()?;
	raw.split(';').find_map(|pair| {
		let (key, value) = pair.trim().split_once('=')?;
		(key == name).then_some(value)
	})
}

pub fn hash_token(token: &str) -> String {
	hex(&Sha256::digest(token.as_bytes()))
}

fn hex(bytes: &[u8]) -> String {
	bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn new_token() -> String {
	let mut bytes = [0u8; 32];
	getrandom::fill(&mut bytes).expect("no randomness");
	hex(&bytes)
}

pub fn verify(password: &str, hash: Option<&str>) -> bool {
	let real = hash.is_some();
	let Ok(parsed) = PasswordHash::new(hash.unwrap_or(DUMMY)) else {
		return false;
	};
	let ok = Scrypt::new().verify_password(password.as_bytes(), &parsed).is_ok();
	real && ok
}

impl FromRequestParts<SqlitePool> for Me {
	type Rejection = Infallible;

	async fn from_request_parts(parts: &mut Parts, pool: &SqlitePool) -> Result<Self, Self::Rejection> {
		let Some(token) = cookie(parts, COOKIE) else {
			return Ok(Me(None));
		};
		let user = db::session_user(pool, &hash_token(token)).await.ok().flatten();
		Ok(Me(user))
	}
}
