use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::{fs, io, io::Write};

const MAX_HISTORY: usize = 100;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    user_name: String,
    tasks: Vec<String>,
    completed: Vec<String>,
}

enum Pane {
    Pending,
    Completed,
}

enum Mode {
    Normal,
    AddTask(String),
    ConfirmDelete,
}

enum Command {
    Quit,
    SwitchPane,
    MoveUp,
    MoveDown,
    EnterAddTask,
    ConfirmAddTask,
    Cancel,
    ConfirmDelete,
    RejectDelete,
    CompleteTask,
    EnterDeletePrompt,
    ToggleHelp,
    TypeChar(char),
    Backspace,
    Undo,
    Redo,
}

fn map_key(key_code: KeyCode, mode: &Mode) -> Option<Command> {
    match mode {
        Mode::Normal => match key_code {
            KeyCode::Char('q') => Some(Command::Quit),
            KeyCode::Char('?') => Some(Command::ToggleHelp),
            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Char('h') => Some(Command::SwitchPane),
            KeyCode::Char('j') | KeyCode::Down => Some(Command::MoveDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Command::MoveUp),
            KeyCode::Char('a') => Some(Command::EnterAddTask),
            KeyCode::Char('c') => Some(Command::CompleteTask),
            KeyCode::Char('d') => Some(Command::EnterDeletePrompt),
            KeyCode::Char('u') => Some(Command::Undo),
            KeyCode::Char('r') => Some(Command::Redo),
            _ => None,
        },
        Mode::AddTask(_) => match key_code {
            KeyCode::Enter => Some(Command::ConfirmAddTask),
            KeyCode::Esc => Some(Command::Cancel),
            KeyCode::Char(c) => Some(Command::TypeChar(c)),
            KeyCode::Backspace => Some(Command::Backspace),
            _ => None,
        },
        Mode::ConfirmDelete => match key_code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(Command::ConfirmDelete),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Command::RejectDelete),
            _ => None,
        },
    }
}

struct App {
    data: Data,
    current_pane: Pane,
    pending_state: ListState,
    completed_state: ListState,
    mode: Mode,
    status_message: Option<String>,
    should_quit: bool,
    show_help: bool,
    undo_stack: Vec<Data>,
    redo_stack: Vec<Data>,
}

impl App {
    fn new(data: Data) -> Self {
        let mut pending_state = ListState::default();
        if !data.tasks.is_empty() {
            pending_state.select(Some(0));
        }
        let mut completed_state = ListState::default();
        if !data.completed.is_empty() {
            completed_state.select(Some(0));
        }
        App {
            data,
            current_pane: Pane::Pending,
            pending_state,
            completed_state,
            mode: Mode::Normal,
            status_message: None,
            should_quit: false,
            show_help: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn selected_index(&self) -> usize {
        match self.current_pane {
            Pane::Pending => self.pending_state.selected().unwrap_or(0),
            Pane::Completed => self.completed_state.selected().unwrap_or(0),
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.data.clone());
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn sync_selection(&mut self) {
        let pending_len = self.data.tasks.len();
        if pending_len == 0 {
            self.pending_state.select(None);
        } else if self.pending_state.selected().unwrap_or(0) >= pending_len {
            self.pending_state.select(Some(pending_len - 1));
        }
        let completed_len = self.data.completed.len();
        if completed_len == 0 {
            self.completed_state.select(None);
        } else if self.completed_state.selected().unwrap_or(0) >= completed_len {
            self.completed_state.select(Some(completed_len - 1));
        }
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.data.clone());
            self.data = prev;
            self.sync_selection();
            self.status_message = Some("Undo.".into());
            serialize_file(&self.data);
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.data.clone());
            self.data = next;
            self.sync_selection();
            self.status_message = Some("Redo.".into());
            serialize_file(&self.data);
        }
    }

    fn move_up(&mut self) {
        match self.current_pane {
            Pane::Pending => {
                let len = self.data.tasks.len();
                if len > 0 {
                    let i = self.pending_state.selected().unwrap_or(0);
                    self.pending_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
            Pane::Completed => {
                let len = self.data.completed.len();
                if len > 0 {
                    let i = self.completed_state.selected().unwrap_or(0);
                    self.completed_state
                        .select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
        }
    }

    fn move_down(&mut self) {
        match self.current_pane {
            Pane::Pending => {
                let len = self.data.tasks.len();
                if len > 0 {
                    let i = self.pending_state.selected().unwrap_or(0);
                    self.pending_state
                        .select(Some(if i + 1 >= len { 0 } else { i + 1 }));
                }
            }
            Pane::Completed => {
                let len = self.data.completed.len();
                if len > 0 {
                    let i = self.completed_state.selected().unwrap_or(0);
                    self.completed_state
                        .select(Some(if i + 1 >= len { 0 } else { i + 1 }));
                }
            }
        }
    }

    fn switch_pane(&mut self) {
        self.current_pane = match self.current_pane {
            Pane::Pending => Pane::Completed,
            Pane::Completed => Pane::Pending,
        };
    }

    fn add_task(&mut self) {
        if let Mode::AddTask(text) = &self.mode {
            let task = text.trim().to_string();
            if !task.is_empty() {
                self.push_undo();
                self.data.tasks.push(task);
                if self.data.tasks.len() == 1 {
                    self.pending_state.select(Some(0));
                }
                self.status_message = Some("Task added.".into());
                serialize_file(&self.data);
            }
        }
        self.mode = Mode::Normal;
    }

    fn complete_task(&mut self) {
        let i = self.pending_state.selected().unwrap_or(0);
        if i < self.data.tasks.len() {
            self.push_undo();
            let task = self.data.tasks.remove(i);
            self.data.completed.push(task);
            if self.data.completed.len() == 1 {
                self.completed_state.select(Some(0));
            }
            if self.data.tasks.is_empty() {
                self.pending_state.select(None);
            } else if i >= self.data.tasks.len() {
                self.pending_state.select(Some(self.data.tasks.len() - 1));
            }
            self.status_message = Some("Task completed.".into());
            serialize_file(&self.data);
        }
    }

    fn delete_task(&mut self) {
        let i = self.selected_index();
        match self.current_pane {
            Pane::Pending => {
                if i < self.data.tasks.len() {
                    self.data.tasks.remove(i);
                    if self.data.tasks.is_empty() {
                        self.pending_state.select(None);
                    } else if i >= self.data.tasks.len() {
                        self.pending_state.select(Some(self.data.tasks.len() - 1));
                    }
                    self.status_message = Some("Task deleted.".into());
                    serialize_file(&self.data);
                }
            }
            Pane::Completed => {
                if i < self.data.completed.len() {
                    self.data.completed.remove(i);
                    if self.data.completed.is_empty() {
                        self.completed_state.select(None);
                    } else if i >= self.data.completed.len() {
                        self.completed_state
                            .select(Some(self.data.completed.len() - 1));
                    }
                    self.status_message = Some("Task deleted.".into());
                    serialize_file(&self.data);
                }
            }
        }
        self.mode = Mode::Normal;
    }

    fn handle_command(&mut self, cmd: Command) {
        self.status_message = None;

        match cmd {
            Command::Quit => self.should_quit = true,
            Command::ToggleHelp => self.show_help = !self.show_help,
            Command::SwitchPane => self.switch_pane(),
            Command::MoveUp => self.move_up(),
            Command::MoveDown => self.move_down(),
            Command::EnterAddTask => self.mode = Mode::AddTask(String::new()),
            Command::ConfirmAddTask => {
                if let Mode::AddTask(text) = &self.mode {
                    let task = text.clone();
                    self.mode = Mode::AddTask(task);
                }
                self.add_task();
            }
            Command::Cancel => self.mode = Mode::Normal,
            Command::ConfirmDelete => self.delete_task(),
            Command::RejectDelete => self.mode = Mode::Normal,
            Command::CompleteTask => {
                if matches!(self.current_pane, Pane::Pending) {
                    self.complete_task();
                }
            }
            Command::EnterDeletePrompt => {
                let has_items = match self.current_pane {
                    Pane::Pending => !self.data.tasks.is_empty(),
                    Pane::Completed => !self.data.completed.is_empty(),
                };
                if has_items {
                    self.mode = Mode::ConfirmDelete;
                }
            }
            Command::TypeChar(c) => {
                if let Mode::AddTask(text) = &self.mode {
                    let mut new_text = text.clone();
                    new_text.push(c);
                    self.mode = Mode::AddTask(new_text);
                }
            }
            Command::Backspace => {
                if let Mode::AddTask(text) = &self.mode {
                    let mut new_text = text.clone();
                    new_text.pop();
                    self.mode = Mode::AddTask(new_text);
                }
            }
        }
    }
}

fn init() -> io::Result<()> {
    let mut file = fs::File::create_new("user_data.txt")?;
    let data_default = r#"
{
  "user_name": "ExampleUser",
  "tasks": [
    "Buy groceries",
    "Finish report",
    "Call John"
  ],
  "completed": [
    "Walk the dog",
    "Read a book"
  ]
}
"#;
    file.write_all(data_default.as_bytes())?;
    Ok(())
}

fn decerialize_file() -> io::Result<Data> {
    let user_data = fs::read_to_string("user_data.txt")?;
    let data: Data = serde_json::from_str(&user_data)?;
    Ok(data)
}

fn serialize_file(data: &Data) {
    if let Ok(buff) = serde_json::to_string(data) {
        let _ = fs::write("user_data.txt", buff);
    }
}

fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1), Constraint::Length(3)])
        .split(frame.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let pending_items: Vec<ListItem> = app
        .data
        .tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if Some(i) == app.pending_state.selected()
                && matches!(app.current_pane, Pane::Pending)
            {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}. {}", i + 1, t)).style(style)
        })
        .collect();

    let completed_items: Vec<ListItem> = app
        .data
        .completed
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if Some(i) == app.completed_state.selected()
                && matches!(app.current_pane, Pane::Completed)
            {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(format!("{}. {}", i + 1, t)).style(style)
        })
        .collect();

    let pending_count = app.data.tasks.len();
    let completed_count = app.data.completed.len();
    let pending_title = format!(" Pending ({pending_count}) ");
    let completed_title = format!(" Completed ({completed_count}) ");

    let pending_list = List::new(pending_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(pending_title)
                .border_style(match app.current_pane {
                    Pane::Pending => Style::default().fg(Color::Cyan),
                    Pane::Completed => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let completed_list = List::new(completed_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(completed_title)
                .border_style(match app.current_pane {
                    Pane::Completed => Style::default().fg(Color::Cyan),
                    Pane::Pending => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut pending_state = app.pending_state.clone();
    let mut completed_state = app.completed_state.clone();

    frame.render_stateful_widget(pending_list, top_chunks[0], &mut pending_state);
    frame.render_stateful_widget(completed_list, top_chunks[1], &mut completed_state);

    let total = pending_count + completed_count;
    let ratio = if total == 0 {
        0.0
    } else {
        completed_count as f64 / total as f64
    };
    let gauge_label = format!("{completed_count}/{total}");
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .label(gauge_label)
        .ratio(ratio);
    frame.render_widget(gauge, chunks[1]);

    let bottom_text = match &app.mode {
        Mode::Normal => {
            format!(
                " [q]uit  [a]dd  [c]omplete  [d]elete  [Tab] switch pane  {}",
                app.status_message.as_deref().unwrap_or("")
            )
        }
        Mode::AddTask(text) => {
            format!(" Enter task: {text}_ (Enter to confirm, Esc to cancel)")
        }
        Mode::ConfirmDelete => {
            " Delete this task? (y/n)".to_string()
        }
    };

    let bottom = Paragraph::new(bottom_text)
        .block(Block::default().borders(Borders::ALL).title(" Commands "))
        .style(Style::default().fg(Color::White));

    frame.render_widget(bottom, chunks[2]);

    if app.show_help {
        let help_area = ratatui::layout::Rect {
            x: frame.area().x + frame.area().width / 4,
            y: frame.area().y + frame.area().height / 3,
            width: (frame.area().width + 1) / 2,
            height: (frame.area().height + 1) / 3,
        };
        let help_text = "\
q      Quit
a      Add task
c      Complete task
d      Delete task
Tab    Switch pane
j/↓    Move down
k/↑    Move up
?      Toggle help
Esc    Cancel";

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(help, help_area);
    }
}

fn run_app(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::prelude::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;
    let result = main_loop(app, &mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn main_loop(app: &mut App, terminal: &mut ratatui::Terminal<ratatui::prelude::CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if let Some(cmd) = map_key(key.code, &app.mode) {
                app.handle_command(cmd);
            }
        }
    }
}

fn main() -> io::Result<()> {
    if init().is_err() {
        // File already exists, carry on
    }

    let data = decerialize_file()?;
    let mut app = App::new(data);
    run_app(&mut app)?;
    Ok(())
}
