#[tokio::main]
async fn main() {
    println!("Hello from XIOIOI!");
    xioioi::router::serve("0.0.0.0:8080").await;
}
