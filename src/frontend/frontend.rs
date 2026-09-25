use axum::{
	Form, Router,
	extract::{Path, State},
	http::{StatusCode, header, request::Parts},
	response::{Html, IntoResponse, Redirect, Response},
	routing::{get, post},
};
use rust_embed::Embed;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::backend::auth::{self, Me};
use crate::backend::db;
use crate::backend::pkg;

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

fn account(me: &Me) -> String {
	match &me.0 {
		None => "<a href=\"/login\" class=\"flex h-full items-center px-3 text-muted hover:text-bright\">Log in</a>".to_string(),
		Some(user) => format!(
			"<span class=\"px-3 text-bright\">{}</span>\n\t<form method=\"post\" action=\"/logout\" class=\"flex h-full\">\n\t\t<button type=\"submit\" class=\"border-0 bg-transparent px-3 text-muted hover:bg-transparent hover:text-bright\">Log out</button>\n\t</form>",
			escape(&user.name)
		),
	}
}

fn fill(src: &str, vars: &[(&str, String)], depth: u8) -> String {
	let mut out = String::new();
	let mut rest = src;

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
		} else if let Some(partial) = load(&format!("partials/{key}.html")).filter(|_| depth < 4) {
			out.push_str(&fill(partial.trim_end(), vars, depth + 1));
		} else {
			out.push_str(&rest[start..start + end + 4]);
		}
		rest = &after[end + 2..];
	}

	out.push_str(rest);
	out
}

fn render(name: &str, me: &Me, vars: &[(&str, String)]) -> Html<String> {
	let src = load(name).unwrap_or_default();
	let mut all = vec![("account", account(me))];
	all.extend(vars.iter().map(|(k, v)| (*k, v.clone())));
	Html(fill(&src, &all, 0))
}

async fn index(me: Me) -> Html<String> {
	render("index.html", &me, &[])
}

async fn todo(me: Me) -> (StatusCode, Html<String>) {
	(StatusCode::NOT_FOUND, render("todo.html", &me, &[]))
}

fn escape(s: &str) -> String {
	s.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&#39;")
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

async fn problemset(me: Me) -> Html<String> {
	render("problemset.html", &me, &[])
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

async fn problem(me: Me, State(pool): State<SqlitePool>, Path((name, hash)): Path<(String, String)>) -> Result<Html<String>, StatusCode> {
	let id = format!("{name}#{hash}");
	let p = db::problem(&pool, &id)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
		.ok_or(StatusCode::NOT_FOUND)?;
	let pkg = pkg::load(&id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	Ok(render("problem.html", &me, &[
		("title", escape(&p.title)),
		("id", escape(&p.id)),
		("time", time(p.time_limit)),
		("memory", memory(p.memory_limit)),
		("doc", pkg.doc),
		("examples", examples(&pkg.examples)),
	]))
}

#[derive(Deserialize)]
struct LoginForm {
	name: String,
	password: String,
}

fn login_page(me: &Me, name: &str, error: &str) -> Html<String> {
	let error = if error.is_empty() {
		String::new()
	} else {
		format!("<div class=\"alert-danger mb-3\">{}</div>", escape(error))
	};
	render("login.html", me, &[("name", escape(name)), ("error", error)])
}

async fn login_form(me: Me) -> Response {
	if me.0.is_some() {
		return Redirect::to("/").into_response();
	}
	login_page(&me, "", "").into_response()
}

async fn login(me: Me, State(pool): State<SqlitePool>, Form(form): Form<LoginForm>) -> Response {
	let user = match db::user_by_name(&pool, form.name.trim()).await {
		Ok(user) => user,
		Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
	};

	let hash = user.as_ref().map(|u| u.password.clone());
	let password = form.password;
	let ok = tokio::task::spawn_blocking(move || auth::verify(&password, hash.as_deref()))
		.await
		.unwrap_or(false);

	let Some(user) = user.filter(|_| ok) else {
		let page = login_page(&me, &form.name, "Wrong login or password.");
		return (StatusCode::UNAUTHORIZED, page).into_response();
	};

	let token = auth::new_token();
	if db::new_session(&pool, &auth::hash_token(&token), user.id).await.is_err() {
		return StatusCode::INTERNAL_SERVER_ERROR.into_response();
	}
	let cookie = format!("{}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}", auth::COOKIE, auth::MAX_AGE);
	([(header::SET_COOKIE, cookie)], Redirect::to("/")).into_response()
}

async fn logout(State(pool): State<SqlitePool>, parts: Parts) -> Response {
	if let Some(token) = auth::cookie(&parts, auth::COOKIE) {
		let _ = db::end_session(&pool, &auth::hash_token(token)).await;
	}
	let cookie = format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0", auth::COOKIE);
	([(header::SET_COOKIE, cookie)], Redirect::to("/")).into_response()
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
		.route("/login", get(login_form).post(login))
		.route("/logout", post(logout))
		.route("/static/{*path}", get(static_file))
		.with_state(pool)
}
