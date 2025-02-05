use std::io;
use task_manager::app::TaskApp;
use task_manager::io::ConsoleIO;
use task_manager::task_manager::TaskManager;

fn main() -> io::Result<()> {
    let task_manager = TaskManager::new("tasks.json")?;
    let console_io = Box::new(ConsoleIO);
    let mut app = TaskApp::new(task_manager, console_io);
    app.run()
}
