use axum::{
    extract::Path, extract::State, http::StatusCode, response::IntoResponse, routing, Json, Router,
};
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
#[openapi(paths(list_tasks, create_task, delete_task))]
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
        .merge(SwaggerUi::new("/").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/tasks", routing::get(list_tasks).post(create_task))
        .route("/tasks/{id}", routing::delete(delete_task))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/tasks",
    responses(
        (status = 200, description = "List available tasks", body = Vec<Task>),
    )
)]
async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let tasks = sqlx::query_as!(Task, "SELECT * from tasks")
        .fetch_all(&state.db_pool)
        .await
        .map(Json);

    match tasks {
        Ok(tasks) => tasks.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error: {}", err),
        )
            .into_response(),
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
) -> impl IntoResponse {
    let task = Task {
        id: "".to_string(),
        title: payload.title,
        completed: payload.completed.unwrap_or(false),
        due_date: payload.due_date,
    };

    let count = sqlx::query_scalar!("SELECT count(*) as count from tasks")
        .fetch_one(&state.db_pool)
        .await;

    match count {
        Err(_) => return StatusCode::BAD_REQUEST,
        Ok(count) => {
            if count.unwrap_or(MAX_TASKS.into()) >= MAX_TASKS.into() {
                return StatusCode::BAD_REQUEST;
            }
        }
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

#[utoipa::path(
    delete,
    path = "/tasks/{id}",
    responses(
        (status = OK, description = "Task deleted"),
        (status = BAD_REQUEST, description = "Invalid input")
    ),
    params(
        ("id" = String, Path, description = "Task ID to delete"),
    )
)]
async fn delete_task(State(state): State<AppState>, Path(id): Path<String>) -> StatusCode {
    let taskid = uuid::Uuid::parse_str(&id);

    if let Err(_) = taskid {
        return StatusCode::BAD_REQUEST;
    }

    let result = sqlx::query("DELETE FROM tasks where id=$1")
        .bind(taskid.unwrap())
        .execute(&state.db_pool)
        .await;

    match result {
        Ok(_) => StatusCode::OK,
        Err(_err) => {
            println!("Unable to delete record {}", _err);
            StatusCode::BAD_REQUEST
        }
    }
}
