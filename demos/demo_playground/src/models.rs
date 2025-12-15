use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub completed: bool,
    pub created_at: String,
}

impl Task {
    pub fn new(id: u32, title: String) -> Self {
        Task {
            id,
            title,
            completed: false,
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        let task = Task::new(1, "Test task".to_string());
        assert_eq!(task.id, 1);
        assert_eq!(task.completed, false);
    }

    #[test]
    fn test_complete_task() {
        let mut task = Task::new(1, "Test task".to_string());
        task.complete();
        assert_eq!(task.completed, true);
    }
}
