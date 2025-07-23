use std::io;

use crate::{project_item::{ProjectItem, ProjectItemType}, screen::Screen};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, Widget}, DefaultTerminal, Frame
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
pub struct App<'a> {
    pub project_tree: Vec<TreeItem<'a, ProjectItem>>,
    tree_state: TreeState<ProjectItem>,
    app_screen: Screen,
    select_idx: u8,
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
            KeyCode::Char('q') => {
                match self.app_screen {
                    Screen::Main => return true,
                    _ => {
                        self.app_screen = Screen::Main; return false
                    },
                }
            },
            KeyCode::Char('j') | KeyCode::Down => {
                match self.app_screen {
                    Screen::Main => {self.tree_state.key_down();},
                    Screen::ProjectMenu => self.move_selection_down(1),
                    Screen::ProjectWorktreeMenu => self.move_selection_down(1),
                    Screen::ProjectDirectoryMenu => {},
                    _ => {},
                };
                return false;
            },
            KeyCode::Char('k') | KeyCode::Up => {
                match self.app_screen {
                    Screen::Main => {self.tree_state.key_up();},
                    Screen::ProjectMenu => self.move_selection_up(1),
                    Screen::ProjectWorktreeMenu => self.move_selection_up(1),
                    Screen::ProjectDirectoryMenu => {},
                    _ => {},
                };
                return false;
            }
            KeyCode::Char(' ') => {
                self.tree_state.toggle_selected();
            }
            KeyCode::Char('x') => {
                self.app_screen = Screen::BranchDelete;
                return false;
            }
            KeyCode::Char('y') => {
                match self.app_screen {
                    Screen::BranchDelete => self.delete_branch(),
                    _ => {}
                }
            }
            KeyCode::Char('n') => {
                match self.app_screen {
                    Screen::BranchDelete | Screen::WorktreeDelete => self.app_screen = Screen::Main,
                    _ => {}
                }
            }
            KeyCode::Enter => {
                let Some(selected_proj) = self.tree_state.selected().last() else {
                    return false;
                };
                match selected_proj.project_type {
                    ProjectItemType::Project => self.app_screen = Screen::ProjectMenu,
                    ProjectItemType::ProjectWorktree => self.app_screen = Screen::ProjectWorktreeMenu,
                    ProjectItemType::ProjectDirectory => self.app_screen = Screen::ProjectDirectoryMenu,
                }
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
        frame.render_stateful_widget(widget, area, &mut self.tree_state);

        let mut block = Block::bordered();
        let Some(selected_proj) = self.tree_state.selected().last() else {
            return;
        };

        match self.app_screen
        {
            Screen::WorktreeDelete | Screen::BranchDelete => {
                let to_delete = if self.app_screen == Screen::BranchDelete {
                    "Branch"
                } else {
                    "Project"
                };
                let paragraph = Paragraph::new(format!("Delete {} [Y/n]?", to_delete))
                    .centered()
                    .block(block);
                let pop_area = popup_area(area, 25, 1);

                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
            },
            Screen::ProjectDirectoryMenu => {
                block = block.title(format!(" Project Directory: {} ", selected_proj.path.file_name().unwrap().to_string_lossy()));
                let paragraph = Paragraph::new(vec![Line::styled(">> Checkout New Worktree", Style::default().add_modifier(Modifier::BOLD))])
                    .left_aligned()
                    .block(block);

                let pop_area = popup_area(area, 25, 1);
                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
            }
            Screen::ProjectWorktreeMenu => {
                block = block.title(format!(" Project Worktree: {} ", selected_proj.path.file_name().unwrap().to_string_lossy()));
                let opts = vec![
                    "New Feat Branch",
                    "Delete Checkout",
                ];

                let fmt_lines = self.opts_to_lines(&opts);

                let paragraph = Paragraph::new(fmt_lines)
                    .left_aligned()
                    .block(block);

                let pop_area = popup_area(area, 25, opts.len() as u16);
                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
            }
            Screen::ProjectMenu => {
                block = block.title(format!(" Project: {} ", selected_proj.path.file_name().unwrap().to_string_lossy()));
                let opts = vec![
                    "Switch To Branch",
                    "Delete Branch",
                ];

                let fmt_lines = self.opts_to_lines(&opts);

                let paragraph = Paragraph::new(fmt_lines)
                    .left_aligned()
                    .block(block);

                let pop_area = popup_area(area, 25, opts.len() as u16);
                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
            }
            _ => {}
        }
    }

    fn opts_to_lines(&self, opts: &Vec<&str>) -> Vec<Line> {
        let mut fmt_lines = vec![];
        for (i, opt) in opts.iter().enumerate() {
            if i == self.select_idx as usize {
                fmt_lines.push(Line::styled(format!(">> {}", opt), Style::default().add_modifier(Modifier::BOLD)));
            }
            else {
                fmt_lines.push(Line::raw(format!("{}", opt)));
            }
        }
        return fmt_lines;
    }

    fn move_selection_down(&mut self, last_idx: u8)
    {
        if self.select_idx == 0 {
            self.select_idx = last_idx;
        }
        else {
            self.select_idx -= 1;
        }
    }

    fn move_selection_up(&mut self, last_idx: u8)
    {
        if self.select_idx >= last_idx {
            self.select_idx = 0;
        }
        else {
            self.select_idx += 1;
        }
    }

    fn delete_branch(&mut self) {
        println!("branch deleted.");
        self.app_screen = Screen::Main;
    }

    fn delete_project(&mut self) {
        println!("project deleted.");
        self.app_screen = Screen::Main;
    }
}

fn popup_area(area: Rect, percent_x: u16, list_items: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(list_items + 2)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
