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

async fn index() -> Html<&'static str> {
	Html(include_str!("web/templates/index.html"))
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
		.route("/static/{*path}", get(static_file))
}
