use crate::io::IO;
use crate::task_manager::{TaskManager, TaskPriority, TaskStatus};
use std::io;

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
            self.io.write_line("5. Exit")?;

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
        self.io
            .write_line("Enter task priority (1: Low, 2:Medium, 3: High): ")?;
        let priority_str = self.io.read_line()?;

        let priority = match priority_str.as_str() {
            "1" => TaskPriority::Low,
            "2" => TaskPriority::Medium,
            "3" => TaskPriority::High,
            _ => TaskPriority::Low,
        };

        match self.add_task_logic(title, description, priority) {
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

    fn add_task_logic(
        &mut self,
        title: String,
        description: String,
        priority: TaskPriority,
    ) -> Result<u32, String> {
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        self.task_manager
            .add_task(title, description, priority)
            .map_err(|e| e.to_string())
    }

    fn handle_list_tasks(&mut self) -> io::Result<()> {
        for task in self.task_manager.list_tasks_sorted_by_priority() {
            self.io.write_line(&format!("\nID: {}", task.id))?;
            self.io.write_line(&format!("Title: {}", task.title))?;
            self.io
                .write_line(&format!("Description: {}", task.description))?;
            self.io.write_line(&format!("Status: {:?}", task.status))?;
            self.io
                .write_line(&format!("Priority: {:?}", task.priority))?;
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
