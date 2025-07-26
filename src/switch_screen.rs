use ratatui::{
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::screen::Screen;

pub struct ScreenSwitcher {}

impl ScreenSwitcher {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct ScreenSwitcherState {
    title: String,
    options: Vec<(String, Screen)>,
    idx: usize,
}

impl ScreenSwitcherState {
    pub fn up(&mut self) {
        self.idx = (self.idx + 1) % self.options.len();
    }

    pub fn down(&mut self) {
        if self.idx == 0 {
            self.idx = self.options.len() - 1;
            return;
        }

        self.idx -= 1;
    }

    pub fn target_screen(&self) -> Screen {
        self.options[self.idx].1
    }

    fn get_fmt_lines(&self) -> Vec<Line> {
        let mut fmt_lines = vec![];
        for (i, opt) in self.options.iter().map(|(o, _)| o).enumerate() {
            if i == self.idx as usize {
                fmt_lines.push(Line::styled(
                    format!(">> {} <<", opt),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            } else {
                fmt_lines.push(Line::raw(format!("{}", opt)));
            }
        }
        return fmt_lines;
    }

    pub fn get_options_count(&self) -> usize {
        self.options.len()
    }
}

impl StatefulWidget for ScreenSwitcher {
    type State = ScreenSwitcherState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let lines = state.get_fmt_lines();

        let paragraph = Paragraph::new(lines)
            .centered()
            .block(Block::bordered().title(state.title.clone()));

        paragraph.render(area, buf);
    }
}

pub struct ScreenSwitcherStateBuilder {
    title: String,
    options: Vec<(String, Screen)>,
}

impl ScreenSwitcherStateBuilder {
    pub fn new(title: String) -> ScreenSwitcherStateBuilder {
        Self {
            title,
            options: vec![],
        }
    }

    pub fn with_option(mut self, text: String, target_screen: Screen) -> Self {
        self.options.push((text, target_screen));
        return self;
    }

    pub fn build(self) -> ScreenSwitcherState {
        ScreenSwitcherState {
            title: self.title,
            options: self.options,
            idx: 0,
        }
    }
}
