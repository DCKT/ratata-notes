mod db;
mod models;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, ToSpan},
    widgets::{Block, List, ListState, Paragraph},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    db::Database,
    models::{Note, NoteList},
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let db = Database::new("notes.db")?;
    let notes = db.get_all_notes()?;
    let mut list_state = ListState::default();

    if !notes.is_empty() {
        list_state.select(Some(0));
    }

    let mut app = App {
        notes: NoteList {
            items: notes,
            state: list_state,
        },
        db,
        current_screen: Screen::List,
        title_input: Input::default(),
        content_input: Input::default(),
        focused_input: FocusedInput::Title,
        should_quit: false,
    };
    ratatui::run(|t| app.run(t))?;

    Ok(())
}

enum Screen {
    List,
    Form,
    ExitConfirm,
}

enum FocusedInput {
    Title,
    Content,
}
enum ListAction {
    MoveUp,
    MoveDown,
    AddNote,
    SelectNote,
    DeleteNote,
    Quit,
}
enum FormAction {
    Save,
    ToggleInput,
    UpdateInput(Event),
    Exit,
}

enum ExitAction {
    Confirm,
    Cancel,
}

enum Action {
    List(ListAction),
    Form(FormAction),
    Exit(ExitAction),
}

struct App {
    db: Database,
    notes: NoteList,
    current_screen: Screen,
    title_input: Input,
    content_input: Input,
    focused_input: FocusedInput,
    should_quit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;
            let event = crossterm::event::read()?;

            if let crossterm::event::Event::Key(key) = event {
                let mut action = self.handle_key(key, event);

                while action.is_some() {
                    action = self.handle_action(action.unwrap());
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.current_screen {
            Screen::List => {
                self.render_list(frame);
            }
            Screen::Form => {
                self.render_form(frame);
            }
            Screen::ExitConfirm => {
                self.render_exit(frame);
            }
        }
    }

    fn handle_key(&mut self, key: event::KeyEvent, event: Event) -> Option<Action> {
        match self.current_screen {
            Screen::List => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Action::List(ListAction::Quit)),
                KeyCode::Char('j') | KeyCode::Down => Some(Action::List(ListAction::MoveDown)),
                KeyCode::Char('k') | KeyCode::Up => Some(Action::List(ListAction::MoveUp)),
                KeyCode::Enter | KeyCode::Char('e') => Some(Action::List(ListAction::SelectNote)),
                KeyCode::Char('a') | KeyCode::Char('i') => Some(Action::List(ListAction::AddNote)),
                KeyCode::Char('d') => Some(Action::List(ListAction::DeleteNote)),
                _ => None,
            },
            Screen::Form => match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => Some(Action::Form(FormAction::Save)),
                (_, KeyCode::Tab) => Some(Action::Form(FormAction::ToggleInput)),
                (_, KeyCode::Esc) => Some(Action::Form(FormAction::Exit)),
                _ => Some(Action::Form(FormAction::UpdateInput(event))),
            },
            Screen::ExitConfirm => match key.code {
                KeyCode::Esc => Some(Action::Exit(ExitAction::Cancel)),
                KeyCode::Char('q') => Some(Action::Exit(ExitAction::Confirm)),
                _ => None,
            },
        }
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::List(list_action) => match list_action {
                ListAction::Quit => {
                    self.current_screen = Screen::ExitConfirm;
                }
                ListAction::MoveUp => {
                    self.notes.state.select_previous();
                }
                ListAction::MoveDown => {
                    self.notes.state.select_next();
                }
                ListAction::AddNote => {
                    self.add_note();
                    self.title_input.reset();
                    self.content_input.reset();
                    self.current_screen = Screen::Form;
                }
                ListAction::DeleteNote => {
                    self.delete_note();
                }
                ListAction::SelectNote => {
                    self.current_screen = Screen::Form;
                    if let Some(index) = self.notes.state.selected() {
                        let current_note = self.notes.items[index].clone();
                        self.title_input = self.title_input.clone().with_value(current_note.title);
                        self.content_input =
                            self.content_input.clone().with_value(current_note.content);
                    }
                }
            },
            Action::Form(form_action) => match form_action {
                FormAction::Save => {
                    self.save_note();
                }
                FormAction::ToggleInput => {
                    self.toggle_input();
                }
                FormAction::UpdateInput(event) => {
                    match self.focused_input {
                        FocusedInput::Title => {
                            self.title_input.handle_event(&event);
                        }
                        FocusedInput::Content => {
                            self.content_input.handle_event(&event);
                        }
                    };
                }
                FormAction::Exit => {
                    self.current_screen = Screen::List;
                }
            },
            Action::Exit(exit_action) => match exit_action {
                ExitAction::Confirm => self.should_quit = true,
                ExitAction::Cancel => self.current_screen = Screen::List,
            },
        }
        None
    }

    fn render_list(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Min(1)])
            .split(frame.area());

        let inner_list_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(layout[0]);

        let block = Block::bordered()
            .title(Line::raw("My Notes").centered())
            .border_set(border::THICK);

        let notes_list_items = self
            .notes
            .items
            .iter()
            .map(|note| note.title.clone())
            .collect::<List>()
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().black().on_white())
            .highlight_symbol(">>")
            .direction(ratatui::widgets::ListDirection::TopToBottom);

        let note_details = self
            .notes
            .state
            .selected()
            .and_then(|selected_index| self.notes.items.get(selected_index))
            .map(|n| Paragraph::new(n.content.as_str()).block(Block::bordered()));

        let help_message = Line::from_iter([
            "Esc/q".bold().yellow(),
            " exit, ".to_span(),
            "e".bold().yellow(),
            " edit, ".to_span(),
            "a".bold().yellow(),
            " add, ".to_span(),
            "d".bold().red(),
            " delete".to_span(),
        ])
        .centered();

        frame.render_widget(help_message, inner_list_layout[1]);
        frame.render_stateful_widget(
            notes_list_items,
            inner_list_layout[0],
            &mut self.notes.state,
        );
        frame.render_widget(note_details, layout[1]);
    }

    fn render_form(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![Constraint::Max(4), Constraint::Min(1)])
            .split(frame.area());

        let inner_content_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Max(1)])
            .split(layout[1]);

        let help_message = Line::from_iter([
            "Esc".bold().yellow(),
            " exit, ".to_span(),
            "Ctrl+S".bold().yellow(),
            " save, ".to_span(),
            "Tab".bold().yellow(),
            " switch input focus.".to_span(),
        ])
        .centered();

        let mut title_input =
            Paragraph::new(self.title_input.value()).style(Style::default().bold());

        let mut content_input = Paragraph::new(self.content_input.value());
        let mut input_block = Block::bordered().title("Title");
        let mut content_block = Block::bordered().title("Content");

        match self.focused_input {
            FocusedInput::Title => {
                input_block = input_block.border_style(Style::new().yellow());
                let width = layout[0].width.max(3) - 3;
                let scroll = self.title_input.visual_scroll(width as usize);
                title_input = title_input.scroll((0, scroll as u16));

                let x = self.title_input.visual_cursor().max(scroll) - scroll + 1;
                frame.set_cursor_position((layout[0].x + x as u16, layout[0].y + 1));
            }
            FocusedInput::Content => {
                content_block = content_block.border_style(Style::new().yellow());
                let width = layout[1].width.max(3) - 3;
                let scroll = self.content_input.visual_scroll(width as usize);
                content_input = content_input.scroll((0, scroll as u16));

                let x = self.content_input.visual_cursor().max(scroll) - scroll + 1;
                frame.set_cursor_position((layout[1].x + x as u16, layout[1].y + 1));
            }
        }

        frame.render_widget(title_input.block(input_block), layout[0]);
        frame.render_widget(content_input.block(content_block), inner_content_layout[0]);
        frame.render_widget(help_message, inner_content_layout[1]);
    }
    fn render_exit(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![Constraint::Max(2), Constraint::Max(2)])
            .split(frame.area());

        let help_message = Line::from_iter([
            "y".bold().yellow(),
            " Yes, ".to_span(),
            "n".bold().yellow(),
            " No, ".to_span(),
        ])
        .centered();

        let title = Paragraph::new("Wanna quit ?").style(Style::default().bold());

        frame.render_widget(title, layout[0]);
        frame.render_widget(help_message, layout[1]);
    }

    fn save_note(&mut self) {
        if let Some(selected_index) = self.notes.state.selected() {
            let updated_note = self
                .db
                .update_note(
                    self.notes.items[selected_index].id,
                    self.title_input.value(),
                    self.content_input.value(),
                )
                .unwrap();
            self.notes.items[selected_index] = updated_note;
        }
    }
    fn toggle_input(&mut self) {
        self.focused_input = match self.focused_input {
            FocusedInput::Title => FocusedInput::Content,
            FocusedInput::Content => FocusedInput::Title,
        };
    }
    fn add_note(&mut self) {
        let new_note = self.db.add_note("New note", "").unwrap();
        self.notes.items.push(new_note);
        self.notes.state.select(Some(self.notes.items.len() - 1));
    }
    fn delete_note(&mut self) {
        if let Some(selected_index) = self.notes.state.selected() {
            self.db
                .delete_note(self.notes.items[selected_index].id)
                .unwrap();
            self.notes.items.remove(selected_index);
            if selected_index != 0 {
                self.notes.state.select(Some(selected_index - 1));
            }
        }
    }
}
