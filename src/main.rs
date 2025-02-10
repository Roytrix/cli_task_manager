use std::error::Error;
use std::io;
use task_manager::task_manager::TaskManager;
use task_manager::tui_app;
use tui_app::TuiApp;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize terminal cleanup on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        crossterm::terminal::disable_raw_mode().unwrap();
        crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        original_hook(panic);
    }));

    // Run with explicit error handling
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        return Err(err.into());
    }
    Ok(())
}

fn run() -> io::Result<()> {
    let task_manager = TaskManager::new("tasks.json")?;
    let mut app = TuiApp::new(task_manager);
    app.run()
}
