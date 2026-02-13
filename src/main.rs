mod db;
mod models;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, ToSpan},
    widgets::{Block, List, ListItem, Paragraph},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{db::Database, models::Note};

enum Screen {
    List,
    Form,
}

enum FocusedInput {
    Title,
    Content,
}

struct App {
    db: Database,
    notes: Vec<Note>,
    current_screen: Screen,
    list_index: usize,
    title_input: Input,
    content_input: Input,
    focused_input: FocusedInput,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let db = Database::new("notes.db")?;
    let notes = db.get_all_notes()?;
    let mut app = App {
        notes,
        db,
        list_index: 0,
        current_screen: Screen::List,
        title_input: Input::default(),
        content_input: Input::default(),
        focused_input: FocusedInput::Title,
    };
    ratatui::run(|t| app.run(t))?;

    Ok(())
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
            let event = crossterm::event::read()?;

            if let crossterm::event::Event::Key(key) = event {
                match self.current_screen {
                    Screen::List => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if self.list_index == self.notes.len() - 1 {
                                self.list_index = 0;
                            } else {
                                self.list_index += 1;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if self.list_index == 0 {
                                self.list_index = self.notes.len() - 1;
                            } else {
                                self.list_index -= 1;
                            }
                        }
                        KeyCode::Enter | KeyCode::Char('e') => {
                            self.current_screen = Screen::Form;
                            let current_note = self.notes[self.list_index].clone();
                            self.title_input =
                                self.title_input.clone().with_value(current_note.title);
                            self.content_input =
                                self.content_input.clone().with_value(current_note.content);
                        }
                        KeyCode::Char('a') => {
                            self.add_note();
                            self.title_input.reset();
                            self.content_input.reset();
                            self.current_screen = Screen::Form;
                        }
                        KeyCode::Char('d') => {
                            self.delete_note();
                        }
                        _ => {}
                    },
                    Screen::Form => match (key.modifiers, key.code) {
                        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                            self.save_note();
                        }
                        (_, KeyCode::Tab) => {
                            self.toggle_input();
                        }
                        (_, KeyCode::Esc) => self.current_screen = Screen::List,
                        _ => {
                            match self.focused_input {
                                FocusedInput::Title => {
                                    self.title_input.handle_event(&event);
                                }
                                FocusedInput::Content => {
                                    self.content_input.handle_event(&event);
                                }
                            };
                        }
                    },
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        match self.current_screen {
            Screen::List => {
                self.render_list(frame);
            }
            Screen::Form => {
                self.render_form(frame);
            }
        }
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

    fn render_list(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Min(1)])
            .split(frame.area());

        let inner_list_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(layout[0]);

        let block = Block::bordered()
            .title("My Notes")
            .border_set(border::THICK);

        let notes_list_items = self.notes.iter().enumerate().map(|(i, note)| {
            let item = ListItem::new(Line::from(note.title.as_str())).bold();

            if i == self.list_index {
                item.black().on_white()
            } else {
                item
            }
        });
        let note_details = self
            .notes
            .get(self.list_index)
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
        frame.render_widget(
            List::new(notes_list_items).block(block),
            inner_list_layout[0],
        );
        frame.render_widget(note_details, layout[1]);
    }

    fn save_note(&mut self) {
        let updated_note = self
            .db
            .update_note(
                self.notes[self.list_index].id,
                self.title_input.value(),
                self.content_input.value(),
            )
            .unwrap();
        self.notes[self.list_index] = updated_note;
    }

    fn toggle_input(&mut self) {
        self.focused_input = match self.focused_input {
            FocusedInput::Title => FocusedInput::Content,
            FocusedInput::Content => FocusedInput::Title,
        };
    }
    fn add_note(&mut self) {
        let new_note = self.db.add_note("New note", "").unwrap();
        self.notes.push(new_note);
        self.list_index = self.notes.len() - 1;
    }
    fn delete_note(&mut self) {
        self.db.delete_note(self.notes[self.list_index].id).unwrap();
        self.notes.remove(self.list_index);
        if self.list_index != 0 {
            self.list_index -= 1;
        }
    }
}
