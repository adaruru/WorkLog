use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub daily_required_hours: f64,
    pub is_active: bool,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub is_builtin: bool,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub title: String,
    pub content: String,
    pub ticket: String,
    pub status_id: i64,
    pub status_name: String,
    pub status_color: String,
    pub work_date: String,
    pub hours: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryRef {
    pub id: i64,
    pub title: String,
    pub ticket: String,
    pub work_date: String,
    pub status_name: String,
    pub status_color: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryDetail {
    pub entry: Entry,
    pub related: Vec<EntryRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInput {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub title: String,
    pub content: String,
    pub ticket: String,
    pub status_id: Option<i64>,
    pub work_date: String,
    pub hours: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserFilter {
    Any,
    Unowned,
    One,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryQuery {
    pub keyword: String,
    pub date_from: String,
    pub date_to: String,
    pub status_ids: Vec<i64>,
    pub user_filter: UserFilter,
    pub user_id: Option<i64>,
    pub sort_field: String,
    pub sort_dir: String,
}

impl Default for EntryQuery {
    fn default() -> Self {
        Self {
            keyword: String::new(),
            date_from: String::new(),
            date_to: String::new(),
            status_ids: Vec::new(),
            user_filter: UserFilter::Any,
            user_id: None,
            sort_field: "work_date".to_string(),
            sort_dir: "desc".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leave {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub leave_date: String,
    pub hours: f64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    pub date: String,
    pub name: String,
    pub is_workday: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintSettings {
    pub anchor_number: Option<i64>,
    pub anchor_start_date: Option<String>,
    pub length_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoursReport {
    pub scope: String,
    pub sprint_number: Option<i64>,
    pub date_from: String,
    pub date_to: String,
    pub required: f64,
    pub filled: f64,
    pub diff: f64,
    pub has_user: bool,
}
