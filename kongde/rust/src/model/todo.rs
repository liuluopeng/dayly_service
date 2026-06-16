#[derive(Debug, Clone, PartialEq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Done,
    Giveup,
    Other(String),
}

impl TodoStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TodoStatus::Pending => "pending",
            TodoStatus::InProgress => "in_progress",
            TodoStatus::Done => "done",
            TodoStatus::Giveup => "giveup",
            TodoStatus::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => TodoStatus::Pending,
            "in_progress" => TodoStatus::InProgress,
            "done" => TodoStatus::Done,
            "giveup" => TodoStatus::Giveup,
            _ => TodoStatus::Other(s.to_string()),
        }
    }
}

pub struct Todo {
    pub id: Option<i64>,
    pub title: String,
    pub steps: String,
    pub status: String,
    pub deadline: i64,
    pub is_repeat: bool,
}
