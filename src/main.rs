use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, ListItem},
};

struct Note {
    title: String,
    content: String,
}

enum Screen {
    List,
    Form,
}

struct App {
    notes: Vec<Note>,
    current_screen: Screen,
    list_index: u8,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut app = App {
        notes: vec![
            Note {
                title: String::from("Hello world"),
                content: String::from("content"),
            },
            Note {
                title: String::from("new title"),
                content: String::from("content"),
            },
            Note {
                title: String::from("new title 2"),
                content: String::from("content"),
            },
        ],
        list_index: 0,
        current_screen: Screen::List,
    };
    ratatui::run(|t| app.run(t))?;
    Ok(())
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                match self.current_screen {
                    Screen::List => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('j') => {
                            if self.list_index == self.notes.len() as u8 - 1 {
                                self.list_index = 0;
                            } else {
                                self.list_index += 1;
                            }
                        }
                        KeyCode::Char('k') => {
                            if self.list_index == 0 {
                                self.list_index = self.notes.len() as u8 - 1;
                            } else {
                                self.list_index -= 1;
                            }
                        }
                        _ => {}
                    },
                    Screen::Form => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(frame.area());

        let block = Block::bordered().title("Notes").border_set(border::THICK);

        let notes_list_items = self.notes.iter().enumerate().map(|(i, note)| {
            let item = ListItem::new(Line::from(note.title.as_str()));

            if i as u8 == self.list_index {
                item.black().on_white()
            } else {
                item
            }
        });

        frame.render_widget(List::new(notes_list_items).block(block), layout[0]);
        frame.render_widget(Block::bordered(), layout[1]);
    }
}
