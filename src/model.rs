use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub completed: bool,
    #[serde(rename = "dueDate")]
    pub due_date: Option<DateTime<Utc>>,
}
