use std::io;

use crate::{project_item::{ProjectItem, ProjectItemType}, screen::{InputMode, Screen}};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize}, text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, Widget}, DefaultTerminal, Frame
};
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
pub struct App<'a> {
    pub project_tree: Vec<TreeItem<'a, ProjectItem>>,
    tree_state: TreeState<ProjectItem>,
    app_screen: Screen,
    select_idx: u8,
    input: Input,
    input_mode: InputMode,
    input_boxes: Vec<String>,
    exit: bool,
}

impl<'a> App<'a> {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let e = event::read()?;
            let should_exit = match self.input_mode {
                InputMode::Standard => {
                    match e {
                        Event::Key(k) => self.handle_key_event(k),
                        _ => false,
                    }
                },
                InputMode::Typing(_) => self.handle_typing(&e),
            };

            if should_exit {
                return Ok(());
            }
        }
        Ok(())
    }

    fn handle_typing(&mut self, e: &Event) -> bool {
        match e {
            Event::Key(k) => {
                match k.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Standard;
                        self.app_screen = Screen::Main;
                        self.input_boxes = vec![];
                        return false;
                    },
                    KeyCode::Enter => return true,
                    KeyCode::Tab => self.next_typing_box(),
                    _ => { self.input.handle_event(e); false },
                }
            }
            _ => return false,
        }
    }

    fn next_typing_box(&mut self) -> bool {
        let InputMode::Typing(idx) = self.input_mode else {
            return false
        };

        self.input_boxes[idx as usize] = self.input.value_and_reset().to_string();

        let new_idx = if idx as usize == self.input_boxes.len() - 1 {
            0
        }
        else {
            idx + 1
        };

        self.input = self.input.clone().with_value(self.input_boxes[new_idx as usize].to_string());

        self.input_mode = InputMode::Typing(new_idx);

        return false;
    }

    fn handle_key_event(&mut self, k: KeyEvent) -> bool {
        match k.code {
            KeyCode::Char('q') | KeyCode::Esc => {
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
                let Some(selected) = self.tree_state.selected().last() else {
                    return false;
                };
                if selected.project_type == ProjectItemType::Project
                {
                    self.app_screen = Screen::BranchDelete;
                }
                else if selected.project_type == ProjectItemType::ProjectWorktree {
                    self.app_screen = Screen::WorktreeDelete
                }
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
                match self.app_screen {
                    Screen::Main => match selected_proj.project_type {
                        ProjectItemType::Project => self.app_screen = Screen::ProjectMenu,
                        ProjectItemType::ProjectWorktree => self.app_screen = Screen::ProjectWorktreeMenu,
                        ProjectItemType::ProjectDirectory => self.app_screen = Screen::ProjectDirectoryMenu,
                    }
                    Screen::ProjectMenu => todo!(),
                    Screen::ProjectWorktreeMenu => todo!(),
                    Screen::ProjectDirectoryMenu => {
                        self.app_screen = Screen::CheckoutNewWorktree;
                        self.input_mode = InputMode::Typing(0);
                        self.input_boxes = vec![String::new(), String::new()];
                    },
                    _ => {}
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
                    "Worktree"
                };
                let paragraph = Paragraph::new(format!("Delete {} [Y/n]?", to_delete))
                    .centered()
                    .block(block);
                let pop_area = popup_list(area, 25, 1);

                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
            },
            Screen::ProjectDirectoryMenu => {
                block = block.title(format!(" Project Directory: {} ", selected_proj.path.file_name().unwrap().to_string_lossy()));
                let paragraph = Paragraph::new(vec![Line::styled(">> Checkout New Worktree", Style::default().add_modifier(Modifier::BOLD))])
                    .centered()
                    .block(block);

                let pop_area = popup_list(area, 25, 1);
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
                    .centered()
                    .block(block);

                let pop_area = popup_list(area, 25, opts.len() as u16);
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
                    .centered()
                    .block(block);

                let pop_area = popup_list(area, 25, opts.len() as u16);
                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
            }
            Screen::CheckoutNewWorktree => {
                block = block.title(format!(" Checkout New Worktree "));
                let pop_area = popup_inputs(area, 50, 20);
                let box_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Length(3),
                        Constraint::Length(pop_area.width - 6),
                        Constraint::Length(3),
                    ])
                    .split(pop_area);
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length((pop_area.height - 6) / 2),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length((pop_area.height - 6) / 2),
                    ])
                    .split(box_area[1]);

                let width = pop_area.width.max(3) - 3;
                let scroll = self.input.visual_scroll(width as usize);

                let InputMode::Typing(box_idx) = self.input_mode else {
                    unreachable!()
                };

                let (input1, input2) = if box_idx == 0 {
                    let i1 = Paragraph::new(self.input.value())
                        .style(Style::new().yellow())
                        .scroll((0, scroll as u16))
                        .block(Block::bordered().title("Git Repo Link"));
                    let i2 = Paragraph::new(self.input_boxes[1].clone())
                        .style(Style::default())
                        .block(Block::bordered().title("Worktree Name (Blank for Git Repo Name)"));
                    (i1, i2)

                } else {
                    let i1 = Paragraph::new(self.input_boxes[0].clone())
                        .style(Style::default())
                        .block(Block::bordered().title("Git Repo Link"));
                    let i2 = Paragraph::new(self.input.value())
                        .style(Style::new().yellow())
                        .scroll((0, scroll as u16))
                        .block(Block::bordered().title("Worktree Name (Blank for Git Repo Name)"));
                    (i1, i2)

                };

                let paragraph = Paragraph::new("")
                    .centered()
                    .block(block);

                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
                frame.render_widget(input1, layout[1]);
                frame.render_widget(input2, layout[2]);

            }
            _ => {}
        }
    }

    fn opts_to_lines(&self, opts: &Vec<&str>) -> Vec<Line> {
        let mut fmt_lines = vec![];
        for (i, opt) in opts.iter().enumerate() {
            if i == self.select_idx as usize {
                fmt_lines.push(Line::styled(format!(">> {} <<", opt), Style::default().add_modifier(Modifier::BOLD)));
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

fn popup_list(area: Rect, percent_x: u16, list_items: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(list_items + 2)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn popup_inputs(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
