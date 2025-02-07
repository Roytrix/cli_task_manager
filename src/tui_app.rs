use crate::task_manager::TaskManager;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::Terminal;

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
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    if self.task_manager.list_tasks_sorted_by_priority().is_empty()
                                    {
                                        0
                                    } else {
                                        self.task_manager.list_tasks_sorted_by_priority().len() - 1
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
                                    if self.task_manager.list_tasks_sorted_by_priority().is_empty()
                                    {
                                        0
                                    } else {
                                        self.task_manager.list_tasks_sorted_by_priority().len() - 1
                                    }
                                } else {
                                    i - 1
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

    fn ui<B: Backend>(&self, f: &mut tui::Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(f.size());

        let tasks: Vec<ListItem> = self
            .task_manager
            .list_tasks_sorted_by_priority()
            .iter()
            .map(|task| {
                ListItem::new(Spans::from(vec![Span::styled(
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
                    Spans::from(vec![
                        Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&task.title),
                    ]),
                    Spans::from(vec![
                        Span::styled(
                            "Description: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&task.description),
                    ]),
                    Spans::from(vec![
                        Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{:?}", task.priority)),
                    ]),
                    Spans::from(vec![
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
