use crate::{backend, frontend};

pub async fn serve(addr: &str) {
	let app = backend::app().merge(frontend::app());
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	println!("xioioi on {addr}");
	axum::serve(listener, app).await.unwrap();
}
