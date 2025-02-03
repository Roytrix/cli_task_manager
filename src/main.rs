use chrono::Local;
use serde_derive::{Deserialize, Serialize};
use serial_test::serial;
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Task {
    id: u32,
    title: String,
    description: String,
    status: TaskStatus,
    created_at: String,
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

    pub fn add_task(&mut self, title: String, description: String) -> io::Result<u32> {
        let task = Task {
            id: self.next_id,
            title,
            description,
            status: TaskStatus::Todo,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
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

    pub fn list_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
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

// IO trait definition
pub trait IO {
    fn read_line(&mut self) -> io::Result<String>;
    fn write_line(&mut self, line: &str) -> io::Result<()>;
}

// Console IO implementation
pub struct ConsoleIO;

impl IO for ConsoleIO {
    fn read_line(&mut self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn write_line(&mut self, line: &str) -> io::Result<()> {
        println!("{}", line);
        Ok(())
    }
}

// Task Application
pub struct TaskApp {
    task_manager: TaskManager,
    io: Box<dyn IO>,
}

impl TaskApp {
    pub fn new(task_manager: TaskManager, io: Box<dyn IO>) -> Self {
        Self { task_manager, io }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.io.write_line("\nTask Manager")?;
            self.io.write_line("1. Add Task")?;
            self.io.write_line("2. List Tasks")?;
            self.io.write_line("3. Update Task Status")?;
            self.io.write_line("4. Delete Task")?;
            self.io.write_line("4. Exit")?;

            match self.io.read_line()?.as_str() {
                "1" => self.handle_add_task()?,
                "2" => self.handle_list_tasks()?,
                "3" => self.handle_update_status()?,
                "4" => self.handle_delete_task()?,
                "5" => break,
                _ => self.io.write_line("Invalid choice!")?,
            }
        }
        Ok(())
    }

    fn handle_add_task(&mut self) -> io::Result<()> {
        self.io.write_line("Enter task title:")?;
        let title = self.io.read_line()?;
        self.io.write_line("Enter task description:")?;
        let description = self.io.read_line()?;

        match self.add_task_logic(title, description) {
            Ok(id) => self.io.write_line(&format!("Task added with ID:{}", id)),
            Err(err) => self.io.write_line(&err),
        }
    }

    fn handle_delete_task(&mut self) -> io::Result<()> {
        self.io.write_line("Enter task ID to delete")?;
        let id_str = self.io.read_line()?;
        let id = id_str.parse::<u32>().unwrap_or(0);

        if self.task_manager.delete_task(id)? {
            self.io.write_line("Task deleted successfully!")?;
        } else {
            self.io.write_line("Task not found!")?;
        }
        Ok(())
    }

    fn add_task_logic(&mut self, title: String, description: String) -> Result<u32, String> {
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        self.task_manager
            .add_task(title, description)
            .map_err(|e| e.to_string())
    }

    fn handle_list_tasks(&mut self) -> io::Result<()> {
        for task in self.task_manager.list_tasks() {
            self.io.write_line(&format!("\nID: {}", task.id))?;
            self.io.write_line(&format!("Title: {}", task.title))?;
            self.io
                .write_line(&format!("Description: {}", task.description))?;
            self.io.write_line(&format!("Status: {:?}", task.status))?;
            self.io
                .write_line(&format!("Created: {}", task.created_at))?;
        }
        Ok(())
    }

    fn handle_update_status(&mut self) -> io::Result<()> {
        self.io.write_line("Enter task ID:")?;
        let id_str = self.io.read_line()?;
        let id = id_str.parse::<u32>().unwrap_or(0);

        self.io
            .write_line("Enter new status (1: Todo, 2: InProgress, 3: Done):")?;
        let status_str = self.io.read_line()?;

        let new_status = match status_str.as_str() {
            "1" => Some(TaskStatus::Todo),
            "2" => Some(TaskStatus::InProgress),
            "3" => Some(TaskStatus::Done),
            _ => None,
        };

        if let Some(status) = new_status {
            if self.task_manager.update_status(id, status)? {
                self.io.write_line("Task updated successfully!")?;
            } else {
                self.io.write_line("Task not found!")?;
            }
        } else {
            self.io.write_line("Invalid status!")?;
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let task_manager = TaskManager::new("tasks.json")?;
    let console_io = Box::new(ConsoleIO);
    let mut app = TaskApp::new(task_manager, console_io);
    app.run()
}

#[cfg(test)]
#[serial]
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
            .add_task(title.clone(), description.clone())
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
            .add_task(title.clone(), description.clone())
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
        let id = task_manager.add_task(title, description).unwrap();

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
        let id = task_manager.add_task(title, description).unwrap();

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
            .add_task("Task 1".to_string(), "Description 1".to_string())
            .unwrap();
        task_manager
            .add_task("Task 2".to_string(), "Description 2".to_string())
            .unwrap();

        let tasks = task_manager.list_tasks();
        delete_test_task_json();

        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn save_and_load_tasks_persists_data() {
        let mut task_manager = setup();

        task_manager
            .add_task("Task 1".to_string(), "Description 1".to_string())
            .unwrap();
        task_manager
            .add_task("Task 2".to_string(), "Description 2".to_string())
            .unwrap();

        let tasks = task_manager.list_tasks();
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
            .add_task("Task 1".to_string(), "Description 1".to_string())
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
}
