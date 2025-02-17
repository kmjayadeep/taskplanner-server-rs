use axum::{routing, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", routing::get(index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap()
}

async fn index() -> &'static str {
    "Hello world"
}
