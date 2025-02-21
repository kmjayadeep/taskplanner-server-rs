use axum::{extract::State, http::StatusCode, routing, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

const MAX_TASKS: i32 = 100;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct Task {
    id: String,
    title: String,
    completed: bool,
    #[serde(rename = "dueDate")]
    due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct CreateTaskInput {
    title: String,
    completed: Option<bool>,
    #[serde(rename = "dueDate")]
    due_date: Option<DateTime<Utc>>,
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool<Postgres>,
}

#[derive(OpenApi)]
#[openapi(paths(list_tasks, create_task))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Unable to connect to DB");

    let state = AppState { db_pool: pool };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", routing::get(index))
        .route("/tasks", routing::get(list_tasks).post(create_task))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "Task Planner v0.1.0"
}

#[utoipa::path(
    get,
    path = "/tasks",
    responses(
        (status = 200, description = "List available tasks", body = Vec<Task>),
    )
)]
async fn list_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let tasks = sqlx::query_as!(Task, "SELECT * from tasks")
        .fetch_all(&state.db_pool)
        .await;

    match tasks {
        Ok(tasks) => Json(tasks),
        Err(err) => {
            println!("Unable to fetch tasks : error {}", err);
            Json(vec![])
        }
    }
}

#[utoipa::path(
    post,
    path = "/tasks",
    responses(
        (status = CREATED, description = "Task created"),
        (status = BAD_REQUEST, description = "Invalid input")
    ),
    request_body = CreateTaskInput
)]
async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskInput>,
) -> StatusCode {
    let task = Task {
        id: "".to_string(),
        title: payload.title,
        completed: payload.completed.unwrap_or(false),
        due_date: payload.due_date,
    };

    let count = sqlx::query_scalar!("SELECT count(*) as count from tasks")
        .fetch_one(&state.db_pool)
        .await
        .unwrap()
        .unwrap();

    // Protection against flooding the DB
    if count >= MAX_TASKS.into() {
        return StatusCode::BAD_REQUEST;
    }

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
