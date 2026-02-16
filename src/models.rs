use ratatui::widgets::ListState;

pub struct NoteList {
    pub items: Vec<Note>,
    pub state: ListState,
}
#[derive(Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
}
