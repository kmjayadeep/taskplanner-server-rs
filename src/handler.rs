use crate::AppState;
use crate::Task;

use axum::{extract::Path, extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const MAX_TASKS: i32 = 100;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTaskInput {
    pub title: String,
    pub completed: Option<bool>,
    #[serde(rename = "dueDate")]
    pub due_date: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "/tasks",
    responses(
        (status = 200, description = "List available tasks", body = Vec<Task>),
    )
)]
pub async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
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
pub async fn create_task(
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
pub async fn delete_task(State(state): State<AppState>, Path(id): Path<String>) -> StatusCode {
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

#[utoipa::path(
    put,
    path = "/tasks/{id}",
    responses(
        (status = OK, description = "Task updated"),
        (status = BAD_REQUEST, description = "Invalid input")
    ),
    params(
        ("id" = String, Path, description = "Task ID to update"),
    ),
    request_body = CreateTaskInput
)]
pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateTaskInput>,
) -> impl IntoResponse {
    let taskid = uuid::Uuid::parse_str(&id);
    if let Err(_) = taskid {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let taskid = taskid.unwrap();

    let query_result = sqlx::query_as!(Task, "SELECT * FROM Tasks WHERE id = $1", taskid)
        .fetch_one(&state.db_pool)
        .await;

    if let Err(_) = query_result {
        return (
            StatusCode::BAD_REQUEST,
            format!("Invalid task id {}", taskid),
        )
            .into_response();
    }

    let task = Task {
        id: "".to_string(),
        title: payload.title,
        completed: payload.completed.unwrap_or(false),
        due_date: payload.due_date,
    };

    let query_result = sqlx::query_as!(
        Task,
        "UPDATE Tasks set title=$1, completed=$2, due_date=$3 WHERE id=$4 RETURNING *",
        task.title,
        task.completed,
        task.due_date,
        taskid,
    )
    .fetch_one(&state.db_pool)
    .await
    .map(Json);

    match query_result {
        Ok(task) => task.into_response(),
        Err(_err) => StatusCode::BAD_REQUEST.into_response(),
    }
}
