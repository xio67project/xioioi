use xioioi::{backend, frontend};

#[tokio::main]
async fn main() {
    println!("Hello from XIOIOI!");
    let addr = "0.0.0.0:8080";
    let pool = backend::db::open().await;
    let app = backend::app(pool.clone()).merge(frontend::app(pool));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("xioioi on {addr}");
    axum::serve(listener, app).await.unwrap();
}
