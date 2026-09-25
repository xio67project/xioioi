use axum::{
	Router,
	extract::{Path, Query, State},
	http::{StatusCode, header},
	response::{Html, IntoResponse, Response},
	routing::get,
};
use rust_embed::Embed;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::db::{self, Order, Problem};
use crate::pkg;

#[derive(Embed)]
#[folder = "src/frontend/web/static/"]
struct Static;

#[derive(Embed)]
#[folder = "src/frontend/web/templates/"]
struct Templates;

fn load(path: &str) -> Option<String> {
	let file = Templates::get(path)?;
	Some(String::from_utf8_lossy(&file.data).into_owned())
}

fn render(name: &str) -> Html<String> {
	render_with(name, &[])
}

fn render_with(name: &str, vars: &[(&str, String)]) -> Html<String> {
	let src = load(name).unwrap_or_default();
	let mut out = String::new();
	let mut rest = src.as_str();

	while let Some(start) = rest.find("%%") {
		out.push_str(&rest[..start]);
		let after = &rest[start + 2..];

		let Some(end) = after.find("%%") else {
			out.push_str(&rest[start..]);
			rest = "";
			break;
		};

		let key = &after[..end];
		if let Some((_, value)) = vars.iter().find(|(k, _)| *k == key) {
			out.push_str(value);
		} else if let Some(partial) = load(&format!("partials/{key}.html")) {
			out.push_str(partial.trim_end());
		} else {
			out.push_str(&rest[start..start + end + 4]);
		}
		rest = &after[end + 2..];
	}

	out.push_str(rest);
	Html(out)
}

async fn index() -> Html<String> {
	render("index.html")
}

async fn todo() -> (StatusCode, Html<String>) {
	(StatusCode::NOT_FOUND, render("todo.html"))
}

fn escape(s: &str) -> String {
	s.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&#39;")
}

fn encode(s: &str) -> String {
	let mut out = String::new();
	for b in s.bytes() {
		if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
			out.push(b as char);
		} else {
			out.push_str(&format!("%{b:02X}"));
		}
	}
	out
}

fn time(ms: i64) -> String {
	if ms % 1000 == 0 {
		format!("{} s", ms / 1000)
	} else {
		format!("{ms} ms")
	}
}

fn memory(kib: i64) -> String {
	if kib % 1024 == 0 {
		format!("{} MB", kib / 1024)
	} else {
		format!("{kib} KB")
	}
}

fn link(id: &str) -> String {
	let (name, hash) = id.split_once('#').unwrap_or((id, ""));
	format!("/p/{}/{}", encode(name), encode(hash))
}

fn row(p: &Problem) -> String {
	format!(
		"<tr>\n\t<td class=\"font-mono whitespace-nowrap\">{}</td>\n\t<td><a href=\"{}\">{}</a></td>\n\t<td class=\"text-center whitespace-nowrap\">{}</td>\n\t<td class=\"text-center whitespace-nowrap\">{}</td>\n</tr>",
		escape(&p.id),
		link(&p.id),
		escape(&p.title),
		time(p.time_limit),
		memory(p.memory_limit),
	)
}

#[derive(Deserialize)]
struct SearchQuery {
	#[serde(default)]
	q: String,
	order_by: Option<String>,
}

async fn problemset(State(pool): State<SqlitePool>, Query(query): Query<SearchQuery>) -> Result<Html<String>, StatusCode> {
	let order = match query.order_by.as_deref() {
		Some("name") => Order::Name,
		_ => Order::Id,
	};
	let problems = db::search(&pool, query.q.trim(), order)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let rows = if problems.is_empty() {
		"<tr><td colspan=\"4\" class=\"text-center text-muted\">No problems found.</td></tr>".to_string()
	} else {
		problems.iter().map(row).collect::<Vec<_>>().join("\n")
	};
	let qs = if query.q.is_empty() {
		String::new()
	} else {
		format!("&amp;q={}", encode(&query.q))
	};

	Ok(render_with("problemset.html", &[("rows", rows), ("q", escape(&query.q)), ("qs", qs)]))
}

fn examples(list: &[pkg::Example]) -> String {
	let mut out = String::new();
	for (i, ex) in list.iter().enumerate() {
		let n = i + 1;
		out.push_str(&format!(
			"<h2 class=\"mt-8 mb-3 text-xl font-bold\">Example {n}</h2>\n<div class=\"grid gap-4 md:grid-cols-2\">\n\t<div>\n\t\t<h3 class=\"mb-2 font-bold\">Input</h3>\n\t\t<pre><code>{}</code></pre>\n\t</div>\n\t<div>\n\t\t<h3 class=\"mb-2 font-bold\">Output</h3>\n\t\t<pre><code>{}</code></pre>\n\t</div>\n</div>\n",
			escape(&ex.input),
			escape(&ex.output),
		));
	}
	out
}

async fn problem(State(pool): State<SqlitePool>, Path((name, hash)): Path<(String, String)>) -> Result<Html<String>, StatusCode> {
	let id = format!("{name}#{hash}");
	let p = db::problem(&pool, &id)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
		.ok_or(StatusCode::NOT_FOUND)?;
	let pkg = pkg::load(&id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	Ok(render_with("problem.html", &[
		("title", escape(&p.title)),
		("id", escape(&p.id)),
		("time", time(p.time_limit)),
		("memory", memory(p.memory_limit)),
		("doc", pkg.doc),
		("examples", examples(&pkg.examples)),
	]))
}

async fn static_file(Path(path): Path<String>) -> Response {
	let Some(file) = Static::get(&path) else {
		return StatusCode::NOT_FOUND.into_response();
	};
	let mime = file.metadata.mimetype().to_string();
	let headers = [(header::CONTENT_TYPE, mime)];
	(headers, file.data).into_response()
}

pub fn app(pool: SqlitePool) -> Router {
	Router::new()
		.route("/", get(index))
		.route("/todo", get(todo))
		.route("/problemset", get(problemset))
		.route("/p/{name}/{hash}", get(problem))
		.route("/static/{*path}", get(static_file))
		.with_state(pool)
}
