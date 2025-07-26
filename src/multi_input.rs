use ratatui::{
    crossterm::event::{Event, KeyCode}, layout::{Constraint, Direction, Layout}, style::{Style, Stylize}, widgets::{Block, Paragraph, StatefulWidget, Widget}
};
use tui_input::{backend::crossterm::EventHandler, Input};

pub struct MultiInput {}

#[derive(Debug)]
pub struct MultiInputState {
    title: String,
    boxes: Vec<InputBox>,
    idx: usize,
}

#[derive(Debug)]
struct InputBox {
    prompt: String,
    handler: Input,
}

impl InputBox {
    fn new(prompt: String) -> Self {
        Self {
            prompt,
            handler: Input::new("".to_string()),
        }
    }
}

impl MultiInputState {
    pub fn new(title: String, input_prompts: Vec<String>) -> Self {
        return MultiInputState {
            title,
            boxes: input_prompts.into_iter().map(InputBox::new).collect(),
            idx: 0,
        };
    }

    pub fn next_box(&mut self) {
        if self.boxes.len() <= 1 {
            return;
        }

        self.idx = (self.idx + 1) % self.boxes.len();
    }

    /// returns true if the event caused this widget to close.
    pub fn handle_event(&mut self, e: &Event) -> bool {
        match e {
            Event::Key(k) => match k.code {
                KeyCode::Esc => {
                    return true;
                }
                KeyCode::Enter => {
                    return true;
                }
                KeyCode::Tab => {
                    self.next_box();
                    return false;
                },
                _ => {
                    self.boxes[self.idx].handler.handle_event(e);
                    false
                }
            },
            _ => return false,
        }
    }

    pub fn get_content_at(&self, idx: usize) -> String {
        self.boxes[idx].handler.value().to_string()
    }
}

impl StatefulWidget for MultiInput {
    type State = MultiInputState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let vert_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(area.width - 6),
                Constraint::Length(3),
            ])
            .split(area);
        let used_height = 3 * state.boxes.len() as u16;
        let mut vert_constraints = vec![Constraint::Length((area.height - used_height) / 2)];
        for _ in 0..state.boxes.len() {
            vert_constraints.push(Constraint::Length(3))
        }
        vert_constraints.push(Constraint::Length((area.height - used_height) / 2));

        let input_layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vert_constraints)
            .split(vert_area[1]);

        let width = area.width.max(3) - 3;

        let paragraph = Paragraph::new("")
            .centered()
            .block(Block::bordered().title(state.title.clone()));
        paragraph.render(area, buf);

        for (i, b) in state.boxes.iter().enumerate() {
            let scroll = b.handler.visual_scroll(width as usize);
            let mut widget = Paragraph::new(b.handler.value()).scroll((0, scroll as u16));
            if i == state.idx as usize {
                widget = widget.style(Style::new().yellow());
            }
            widget = widget.block(Block::bordered().title(state.boxes[i].prompt.to_string()));
            widget.render(input_layouts[i + 1], buf);
        }
    }
}
