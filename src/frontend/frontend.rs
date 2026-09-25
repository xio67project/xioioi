use axum::{
	Router,
	extract::Path,
	http::{StatusCode, header},
	response::{Html, IntoResponse, Response},
	routing::get,
};
use rust_embed::Embed;

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
		match load(&format!("partials/{key}.html")) {
			Some(partial) => out.push_str(partial.trim_end()),
			None => out.push_str(&rest[start..start + end + 4]),
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

async fn static_file(Path(path): Path<String>) -> Response {
	let Some(file) = Static::get(&path) else {
		return StatusCode::NOT_FOUND.into_response();
	};
	let mime = file.metadata.mimetype().to_string();
	let headers = [(header::CONTENT_TYPE, mime)];
	(headers, file.data).into_response()
}

pub fn app() -> Router {
	Router::new()
		.route("/", get(index))
		.route("/todo", get(todo))
		.route("/static/{*path}", get(static_file))
}
