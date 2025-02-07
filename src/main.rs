use std::io;
use task_manager::task_manager::TaskManager;
use task_manager::tui_app;
use tui_app::TuiApp;

fn main() -> io::Result<()> {
    let task_manager = TaskManager::new("tasks.json")?;
    let mut app = TuiApp::new(task_manager);
    app.run()
}
