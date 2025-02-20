use axum::{extract::State, http::StatusCode, routing, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    completed: bool,
    #[serde(rename = "dueDate")]
    due_date: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTaskInput {
    title: String,
    completed: Option<bool>,
    #[serde(rename = "dueDate")]
    due_date: Option<i32>,
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Unable to connect to DB");

    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route("/", routing::get(index))
        .route("/tasks", routing::get(list_tasks).post(create_task))
        .with_state(state);

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

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskInput>,
) -> StatusCode {
    let task = Task {
        id: "".to_string(),
        title: payload.title,
        completed: payload.completed.unwrap_or(false),
        due_date: payload.due_date.unwrap_or(0),
    };

    let task = sqlx::query_as!(
        Task,
        "INSERT INTO tasks (title, completed, due_date) VALUES ($1, $2, $3) RETURNING id, title, completed, due_date",
        task.title,
        task.completed,
        task.due_date,
    ).fetch_one(&state.db_pool).await;

    match task {
        Ok(_task) => StatusCode::CREATED,
        Err(_err) => StatusCode::BAD_REQUEST,
    }
}
