use axum::{routing, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    completed: bool,
    #[serde(rename = "dueDate")]
    due_date: u32,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", routing::get(index))
        .route("/tasks", routing::get(list_tasks));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "Hello world"
}

async fn list_tasks() -> Json<Vec<Task>> {
    let todos = vec![Task {
        id: Uuid::new_v4().to_string(),
        title: String::from("test"),
        completed: false,
        due_date: 0,
    }];

    Json(todos)
}
