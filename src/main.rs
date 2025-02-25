mod handler;
mod model;
use axum::{routing, Router};
use sqlx::{Pool, Postgres};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handler::*;
use crate::model::*;

#[derive(Clone)]
struct AppState {
    db_pool: Pool<Postgres>,
}

#[derive(OpenApi)]
#[openapi(paths(list_tasks, create_task, delete_task, update_task))]
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
        .route("/tasks/{id}", routing::delete(delete_task).put(update_task))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
