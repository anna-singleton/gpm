use std::io;

use crate::{project_item::ProjectItem, screen::Screen};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}, widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, Widget}, DefaultTerminal, Frame
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
pub struct App<'a> {
    pub project_tree: Vec<TreeItem<'a, ProjectItem>>,
    state: TreeState<ProjectItem>,
    exit: bool,
}

impl<'a> App<'a> {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            if match event::read()? {
                Event::Key(k) => self.handle_key_event(k),
                _ => false
            }
            {
                return Ok(());
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, k: KeyEvent) -> bool {
        match k.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.key_down();
            },
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.key_up();
            }
            KeyCode::Char(' ') => {
                self.state.toggle_selected();
            }
            _ => return false
        };
        return false;
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let widget = Tree::new(&self.project_tree)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title("Projects"),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}
