use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::Path;
use chrono::Local;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    description: String,
    status: TaskStatus,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

struct TaskManager {
    tasks: HashMap<u32, Task>,
    next_id: u32,
    file_path: String,
}

impl TaskManager {
    fn new(file_path: &str) -> io::Result<Self> {
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

    fn add_task(&mut self, title: String, description: String) -> io::Result<u32> {
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

    fn update_status(&mut self, id: u32, status: TaskStatus) -> io::Result<bool> {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.status = status;
            self.save_tasks()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn list_tasks(&self) -> Vec<&Task> {
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

fn main() -> io::Result<()> {
    let mut task_manager = TaskManager::new("tasks.json")?;

    loop {
        println!("\nTask Manager");
        println!("1. Add Task");
        println!("2. List Tasks");
        println!("3. Update Task Status");
        println!("4. Exit");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => {
                println!("Enter task title:");
                let mut title = String::new();
                io::stdin().read_line(&mut title)?;

                println!("Enter task description:");
                let mut description = String::new();
                io::stdin().read_line(&mut description)?;

                let id = task_manager.add_task(
                    title.trim().to_string(),
                    description.trim().to_string()
                )?;
                println!("Task added with ID: {}", id);
            }
            "2" => {
                for task in task_manager.list_tasks() {
                    println!("\nID: {}", task.id);
                    println!("Title: {}", task.title);
                    println!("Description: {}", task.description);
                    println!("Status: {:?}", task.status);
                    println!("Created: {}", task.created_at);
                }
            }
            "3" => {
                println!("Enter task ID:");
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str)?;
                let id = id_str.trim().parse::<u32>().unwrap_or(0);

                println!("Enter new status (1: Todo, 2: InProgress, 3: Done):");
                let mut status_str = String::new();
                io::stdin().read_line(&mut status_str)?;

                let new_status = match status_str.trim() {
                    "1" => TaskStatus::Todo,
                    "2" => TaskStatus::InProgress,
                    "3" => TaskStatus::Done,
                    _ => continue,
                };

                if task_manager.update_status(id, new_status)? {
                    println!("Task updated successfully!");
                } else {
                    println!("Task not found!");
                }
            }
            "4" => break,
            _ => println!("Invalid choice!"),
        }
    }

    Ok(())
}