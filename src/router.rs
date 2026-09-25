use crate::{backend, db, frontend};

pub async fn serve(addr: &str) {
	let pool = db::open().await;
	let app = backend::app(pool).merge(frontend::app());
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	println!("xioioi on {addr}");
	axum::serve(listener, app).await.unwrap();
}
