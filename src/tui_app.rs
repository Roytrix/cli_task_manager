use crate::task_manager::TaskManager;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*, Terminal};
use std::io;

pub struct TuiApp {
    task_manager: TaskManager,
    list_state: ListState,
}

impl TuiApp {
    pub fn new(task_manager: TaskManager) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            task_manager,
            list_state,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui::<B>(f))?;
            let tasks = self.task_manager.list_tasks_sorted_by_priority();

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    if tasks.is_empty() {
                                        0
                                    } else {
                                        tasks.len() - 1
                                    }
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    if tasks.is_empty() {
                                        0
                                    } else {
                                        tasks.len() + 1
                                    }
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        // Handle task selection
                    }
                    _ => {}
                }
            }
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame) {
        let size = Frame::area(f);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(size);

        let block = Block::default().title("Task Manager").borders(Borders::ALL);
        f.render_widget(block, size);
        if self.task_manager.list_tasks_sorted_by_priority().is_empty() {
            let no_tasks = Paragraph::new("No tasks available")
                .block(Block::default().borders(Borders::ALL).title("Tasks"));
            f.render_widget(no_tasks, chunks[0]);
            return;
        }

        let tasks: Vec<ListItem> = self
            .task_manager
            .list_tasks_sorted_by_priority()
            .iter()
            .map(|task| {
                ListItem::new(Line::from(vec![Span::styled(
                    task.title.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                )]))
            })
            .collect();

        let tasks_list = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .highlight_style(Style::default().bg(Color::LightGreen))
            .highlight_symbol(">> ");

        f.render_stateful_widget(tasks_list, chunks[0], &mut self.list_state.clone());

        if let Some(selected) = self.list_state.selected() {
            let tasks = self.task_manager.list_tasks_sorted_by_priority();
            if !tasks.is_empty() {
                let task = &tasks[selected];
                let task_detail = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&task.title),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "Description: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&task.description),
                    ]),
                    Line::from(vec![
                        Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{:?}", task.priority)),
                    ]),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{:?}", task.status)),
                    ]),
                ])
                    .block(Block::default().borders(Borders::ALL).title("Task Details"));

                f.render_widget(task_detail, chunks[1]);
            }
        }
    }
}
