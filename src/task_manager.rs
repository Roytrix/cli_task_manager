use chrono::Local;
use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Task {
    pub(crate) id: u32,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) status: TaskStatus,
    pub(crate) created_at: String,
    pub(crate) priority: TaskPriority,
}

pub struct TaskManager {
    tasks: HashMap<u32, Task>,
    next_id: u32,
    file_path: String,
}

impl TaskManager {
    pub fn new(file_path: &str) -> io::Result<Self> {
        let mut task_manager = TaskManager {
            tasks: HashMap::new(),
            next_id: 1,
            file_path: file_path.to_string(),
        };

        if Path::new(file_path).exists() {
            task_manager.load_tasks()?;
        }

        Ok(task_manager)
    }

    pub fn add_task(
        &mut self,
        title: String,
        description: String,
        priority: TaskPriority,
    ) -> io::Result<u32> {
        let task = Task {
            id: self.next_id,
            title,
            description,
            status: TaskStatus::Todo,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            priority,
        };

        self.tasks.insert(task.id, task);
        self.next_id += 1;
        self.save_tasks()?;
        Ok(self.next_id - 1)
    }

    pub fn delete_task(&mut self, id: u32) -> io::Result<bool> {
        if self.tasks.remove(&id).is_some() {
            self.save_tasks()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn update_status(&mut self, id: u32, status: TaskStatus) -> io::Result<bool> {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.status = status;
            self.save_tasks()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_tasks_sorted_by_priority(&self) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self.tasks.values().collect();
        tasks.sort_by(|a, b| a.priority.cmp(&b.priority));
        tasks
    }

    fn save_tasks(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.tasks)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn load_tasks(&mut self) -> io::Result<()> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        self.tasks = serde_json::from_reader(reader)?;
        self.next_id = self.tasks.keys().max().map_or(1, |max| max + 1);
        Ok(())
    }
}

#[cfg(test)]
#[serial_test::serial]
mod tests {
    use super::*;
    use std::fs::Permissions;
    use std::time::Duration;
    use std::{fs, thread};

    fn setup() -> TaskManager {
        let file_path = "test_tasks.json";
        TaskManager::new(file_path).unwrap()
    }

    fn delete_test_task_json() {
        let file_path = "test_tasks.json";
        while Path::new(file_path).exists() {
            let _ = fs::remove_file(file_path);
            thread::sleep(Duration::from_secs(1));
        }
        println!("file deleted");
    }

    #[test]
    fn add_task_with_valid_data() {
        let mut task_manager = setup();

        let title = "Test Task".to_string();
        let description = "This is a test task".to_string();
        let id = task_manager
            .add_task(title.clone(), description.clone(), TaskPriority::Low)
            .unwrap();

        let task = task_manager.tasks.get(&id).unwrap();
        delete_test_task_json();

        assert_eq!(task.title, title);
        assert_eq!(task.description, description);
        assert_eq!(task.status, TaskStatus::Todo);
    }
    #[test]
    fn add_task_with_empty_description() {
        let mut task_manager = setup();

        let title = "Test Task".to_string();
        let description = "".to_string();
        let id = task_manager
            .add_task(title.clone(), description.clone(), TaskPriority::Low)
            .unwrap();

        let task = task_manager.tasks.get(&id).unwrap();
        delete_test_task_json();

        assert_eq!(task.title, title);
        assert_eq!(task.description, description);
        assert_eq!(task.status, TaskStatus::Todo);
    }

    #[test]
    fn delete_existing_task() {
        let mut task_manager = setup();

        let title = "Test Task".to_string();
        let description = "This is a test task".to_string();
        let id = task_manager
            .add_task(title, description, TaskPriority::Low)
            .unwrap();

        let deleted = task_manager.delete_task(id).unwrap();
        delete_test_task_json();

        assert!(deleted);
        assert!(!task_manager.tasks.contains_key(&id))
    }

    #[test]
    fn update_status_of_existing_task() {
        let mut task_manager = setup();

        let title = "Test Task".to_string();
        let description = "This is a test task".to_string();
        let id = task_manager
            .add_task(title, description, TaskPriority::Low)
            .unwrap();

        let updated = task_manager
            .update_status(id, TaskStatus::InProgress)
            .unwrap();
        assert!(updated);

        let task = task_manager.tasks.get(&id).unwrap();
        delete_test_task_json();

        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[test]
    fn update_status_of_nonexistent_task() {
        let mut task_manager = setup();

        let result = task_manager.update_status(999, TaskStatus::InProgress);
        delete_test_task_json();

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn list_tasks_returns_all_tasks() {
        let mut task_manager = setup();

        task_manager
            .add_task(
                "Task 1".to_string(),
                "Description 1".to_string(),
                TaskPriority::Low,
            )
            .unwrap();
        task_manager
            .add_task(
                "Task 2".to_string(),
                "Description 2".to_string(),
                TaskPriority::Low,
            )
            .unwrap();

        let tasks = task_manager.list_tasks_sorted_by_priority();
        delete_test_task_json();

        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn save_and_load_tasks_persists_data() {
        let mut task_manager = setup();

        task_manager
            .add_task(
                "Task 1".to_string(),
                "Description 1".to_string(),
                TaskPriority::Low,
            )
            .unwrap();
        task_manager
            .add_task(
                "Task 2".to_string(),
                "Description 2".to_string(),
                TaskPriority::Low,
            )
            .unwrap();

        let tasks = task_manager.list_tasks_sorted_by_priority();
        delete_test_task_json();

        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn load_tasks_with_invalid_json() {
        let mut task_manager = setup();

        let file_path = "test_tasks.json";
        fs::write(file_path, "invalid json").unwrap();

        let result = task_manager.load_tasks();
        delete_test_task_json();

        assert!(result.is_err());
    }

    #[test]
    fn save_tasks_with_read_only_file() {
        let file_path = "test_tasks.json";

        let mut task_manager = setup();

        task_manager
            .add_task(
                "Task 1".to_string(),
                "Description 1".to_string(),
                TaskPriority::Low,
            )
            .unwrap();

        // Make the file read-only
        let metadata = fs::metadata(file_path).unwrap();
        let mut perms = metadata.permissions();
        perms.set_readonly(true);
        fs::set_permissions(file_path, perms.clone()).unwrap();

        let result = task_manager.save_tasks();

        Permissions::set_readonly(&mut perms, false);
        fs::set_permissions(file_path, perms).unwrap();
        delete_test_task_json();
        assert!(result.is_err());
    }

    #[test]
    fn add_task_with_priority() {
        let mut task_manager = setup();

        let title = "Test Task".to_string();
        let description = "This is a test task".to_string();
        let priority = TaskPriority::High;
        let id = task_manager
            .add_task(title.clone(), description.clone(), priority)
            .unwrap();

        let task = task_manager.tasks.get(&id).unwrap();
        delete_test_task_json();

        assert_eq!(task.title, title);
        assert_eq!(task.description, description);
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, priority);
    }

    #[test]
    fn list_tasks_sorted_by_priority() {
        let mut task_manager = setup();

        task_manager
            .add_task(
                "Task 1".to_string(),
                "Description 1".to_string(),
                TaskPriority::Medium,
            )
            .unwrap();
        task_manager
            .add_task(
                "Task 2".to_string(),
                "Description 2".to_string(),
                TaskPriority::High,
            )
            .unwrap();
        task_manager
            .add_task(
                "Task 3".to_string(),
                "Description 3".to_string(),
                TaskPriority::Low,
            )
            .unwrap();

        let tasks = task_manager.list_tasks_sorted_by_priority();
        delete_test_task_json();

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].priority, TaskPriority::Low);
        assert_eq!(tasks[1].priority, TaskPriority::Medium);
        assert_eq!(tasks[2].priority, TaskPriority::High);
    }
}
