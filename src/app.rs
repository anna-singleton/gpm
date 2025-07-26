use std::{
    fs::File,
    io::{self, Write},
    process::{Command, Output},
    str::from_utf8,
};

use crate::{
    config::Config,
    multi_input::{MultiInput, MultiInputState},
    project_item::{ProjectItem, ProjectItemType},
    screen::Screen,
    switch_screen::{ScreenSwitcher, ScreenSwitcherState, ScreenSwitcherStateBuilder},
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
pub struct App<'a> {
    pub project_tree: Vec<TreeItem<'a, ProjectItem>>,
    tree_state: TreeState<ProjectItem>,
    pub app_screen: Screen,
    input_state: Option<MultiInputState>,
    screen_switch_state: Option<ScreenSwitcherState>,
    summary_text: Vec<String>,
}

impl<'a> App<'a> {
    pub fn run(&'a mut self, terminal: &mut DefaultTerminal) -> io::Result<bool> {
        loop {
            self.initialise_screen();
            terminal.draw(|frame| self.draw(frame))?;
            let e = event::read()?;
            if self.app_screen == Screen::Summary {
                match e {
                    Event::Key(k) => match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(false);
                        }
                        _ => return Ok(true),
                    },
                    _ => continue,
                }
            }
            let propagate = if let Some(ref mut s) = self.input_state {
                s.handle_event(&e)
            } else {
                true
            };
            if propagate {
                let should_exit = match e {
                    Event::Key(k) => self.handle_key_event(k),
                    _ => false,
                };
                if should_exit {
                    return Ok(false);
                }
            }
        }
    }

    fn initialise_screen(&mut self) {
        match self.app_screen {
            Screen::Main => {
                self.input_state = None;
                self.screen_switch_state = None;
            }
            Screen::WorktreeCreate => {
                if self.input_state.is_some() {
                    return;
                }
                self.input_state = Some(MultiInputState::new(
                    " Create New Branch as Worktree ".to_string(),
                    vec![
                        "Branch Name".to_string(),
                        "Directory Name (blank for default)".to_string(),
                    ],
                ));
            }
            Screen::WorktreeRepoCreate => {
                if self.input_state.is_some() {
                    return;
                }
                self.input_state = Some(MultiInputState::new(
                    " Create new Worktree Branch ".to_string(),
                    vec![
                        "Repo Link".to_string(),
                        "Directory Name (blank for default)".to_string(),
                    ],
                ));
            }
            Screen::NonWorktreeRepoCreate => {
                if self.input_state.is_some() {
                    return;
                }
                self.input_state = Some(MultiInputState::new(
                    " Create Non-Worktree Repo ".to_string(),
                    vec![
                        "Repo Link".to_string(),
                        "Directory Name (blank for default)".to_string(),
                    ],
                ));
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, k: KeyEvent) -> bool {
        match k.code {
            KeyCode::Esc => match self.app_screen {
                Screen::WorktreeRepoCreate | Screen::WorktreeCreate => {
                    self.input_state = None;
                    self.app_screen = Screen::Main;
                    return false;
                }
                Screen::ScreenSwitchMenu => {
                    self.screen_switch_state = None;
                    self.app_screen = Screen::Main;
                    return false;
                }
                Screen::Main => return true,
                _ => {
                    self.app_screen = Screen::Main;
                    return false;
                }
            },
            KeyCode::Char('q') => match self.app_screen {
                Screen::ScreenSwitchMenu => {
                    self.screen_switch_state = None;
                    self.app_screen = Screen::Main;
                    return false;
                }
                Screen::Main => return true,
                _ => {
                    self.app_screen = Screen::Main;
                    return false;
                }
            },
            KeyCode::Char('j') | KeyCode::Down => {
                match self.app_screen {
                    Screen::Main => {
                        self.tree_state.key_down();
                    }
                    Screen::ScreenSwitchMenu => {
                        if let Some(ref mut s) = self.screen_switch_state {
                            s.down();
                        }
                    }
                    _ => {}
                };
                return false;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.app_screen {
                    Screen::Main => {
                        self.tree_state.key_up();
                    }
                    Screen::ScreenSwitchMenu => {
                        if let Some(ref mut s) = self.screen_switch_state {
                            s.up();
                        }
                    }
                    _ => {}
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
                if selected.project_type == ProjectItemType::Worktree {
                    self.app_screen = Screen::WorktreeDelete;
                } else if selected.project_type == ProjectItemType::WorktreeRepo {
                    self.app_screen = Screen::WorktreeRepoDelete;
                } else if selected.project_type == ProjectItemType::NonWorktreeRepo {
                    self.app_screen = Screen::NonWorktreeRepoDelete;
                }
                return false;
            }
            KeyCode::Char('y') => match self.app_screen {
                Screen::WorktreeDelete => self.delete_worktree(),
                Screen::WorktreeRepoDelete => self.delete_repo(),
                Screen::NonWorktreeRepoDelete => self.delete_repo(),
                _ => {}
            },
            KeyCode::Char('n') => match self.app_screen {
                Screen::WorktreeDelete
                | Screen::WorktreeRepoDelete
                | Screen::NonWorktreeRepoDelete => self.app_screen = Screen::Main,
                _ => {}
            },
            KeyCode::Enter => {
                let Some(selected_proj) = self.tree_state.selected().last() else {
                    return false;
                };
                match self.app_screen {
                    Screen::Main => match selected_proj.project_type {
                        ProjectItemType::NonWorktreeRepo => {
                            self.screen_switch_state = Some(
                                ScreenSwitcherStateBuilder::new(" Project Menu ".to_string())
                                    .with_option(
                                        "Delete Project".to_string(),
                                        Screen::NonWorktreeRepoDelete,
                                    )
                                    .build(),
                            );
                            self.app_screen = Screen::ScreenSwitchMenu
                        }
                        ProjectItemType::Worktree => {
                            self.screen_switch_state = Some(
                                ScreenSwitcherStateBuilder::new(" Project Menu ".to_string())
                                    .with_option(
                                        "Delete Project".to_string(),
                                        Screen::WorktreeDelete,
                                    )
                                    .build(),
                            );
                            self.app_screen = Screen::ScreenSwitchMenu
                        }
                        ProjectItemType::WorktreeRepo => {
                            self.screen_switch_state = Some(
                                ScreenSwitcherStateBuilder::new(
                                    " Project Worktree Menu ".to_string(),
                                )
                                .with_option(
                                    "New Branch As Worktree".to_string(),
                                    Screen::WorktreeCreate,
                                )
                                .with_option(
                                    "Delete Worktree".to_string(),
                                    Screen::WorktreeRepoDelete,
                                )
                                .build(),
                            );
                            self.app_screen = Screen::ScreenSwitchMenu
                        }
                        ProjectItemType::ProjectDirectory => {
                            self.screen_switch_state = Some(
                                ScreenSwitcherStateBuilder::new(
                                    " Project Worktree Menu ".to_string(),
                                )
                                .with_option(
                                    "Checkout New Repo - Worktree Mode".to_string(),
                                    Screen::WorktreeRepoCreate,
                                )
                                .with_option(
                                    "Checkout New Repo - Non Worktree Mode".to_string(),
                                    Screen::NonWorktreeRepoCreate,
                                )
                                .build(),
                            );
                            self.app_screen = Screen::ScreenSwitchMenu
                        }
                    },
                    Screen::WorktreeCreate => {
                        self.checkout_new_worktree();
                        return false;
                    }
                    Screen::WorktreeRepoCreate => {
                        self.checkout_new_worktree_repo();
                        return false;
                    }
                    Screen::NonWorktreeRepoCreate => {
                        self.checkout_new_non_worktree_repo();
                        return false;
                    }
                    Screen::ScreenSwitchMenu => {
                        if let Some(ref state) = self.screen_switch_state {
                            self.app_screen = state.target_screen();
                        }
                        self.screen_switch_state = None;
                    }
                    _ => {}
                }
            }
            _ => return false,
        };
        return false;
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        if self.app_screen == Screen::Summary {
            frame.render_widget(Clear, area);
            let summary_area = popup_inputs(area, 90, 90);
            let mut lines = vec![Line::styled(
                "Execution Summary",
                Style::default().add_modifier(Modifier::BOLD),
            )];
            for l in self.summary_text.iter() {
                lines.push(Line::default());
                lines.push(Line::raw(l.clone()))
            }
            lines.push(Line::styled(
                "Press q / Escape to exit, or any other key to continue editing.",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            let paragraph = Paragraph::new(lines).centered().block(Block::bordered());

            frame.render_widget(paragraph, summary_area);
            return;
        }

        let widget = Tree::new(&self.project_tree)
            .expect("all item identifiers are unique")
            .block(Block::bordered().title("Projects"))
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

        match self.app_screen {
            Screen::WorktreeRepoDelete | Screen::WorktreeDelete | Screen::NonWorktreeRepoDelete => {
                let to_delete = match self.app_screen {
                    Screen::NonWorktreeRepoDelete => "Non-Worktree Repo",
                    Screen::WorktreeRepoDelete => "Worktree Repo",
                    Screen::WorktreeDelete => "Worktree",
                    _ => unreachable!(),
                };
                let paragraph = Paragraph::new(format!("Delete {} [Y/n]?", to_delete))
                    .centered()
                    .block(Block::bordered());
                let pop_area = popup_list(area, 25, 1);

                frame.render_widget(Clear, pop_area);
                frame.render_widget(paragraph, pop_area);
            }
            Screen::ScreenSwitchMenu => {
                if let Some(ref mut state) = self.screen_switch_state {
                    let w = ScreenSwitcher::new();
                    let area = popup_list(area, 50, state.get_options_count() as u16);
                    w.render(area, frame.buffer_mut(), state);
                }
            }
            Screen::WorktreeRepoCreate | Screen::WorktreeCreate | Screen::NonWorktreeRepoCreate => {
                if let Some(ref mut state) = self.input_state {
                    let w = MultiInput {};
                    let pop_area = popup_inputs(area, 50, 20);
                    w.render(pop_area, frame.buffer_mut(), state);
                }
            }
            _ => {}
        }
    }

    fn get_selected_pt_item(&self) -> Option<ProjectItem> {
        self.tree_state.selected().last().cloned()
    }

    fn delete_worktree(&mut self) {
        let Some(wt) = self.get_selected_pt_item() else {
            return;
        };
        let wt_name = wt.path.file_name().unwrap();
        let output = Command::new("git")
            .current_dir(wt.path.parent().unwrap())
            .arg("worktree")
            .arg("remove")
            .arg(wt_name)
            .output()
            .expect("Failed to start 'git' process");

        self.generate_cmd_summary(
            &format!("Deleting Worktree {}", wt_name.to_string_lossy()),
            output,
        );

        self.app_screen = Screen::Summary;
    }

    fn delete_repo(&mut self) {
        let Some(repo) = self.get_selected_pt_item() else {
            return;
        };
        let repo_name = repo.path.file_name().unwrap();

        let output = Command::new("rm")
            .current_dir(&repo.path.parent().unwrap())
            .arg("-rf")
            .arg(repo_name)
            .output()
            .expect("Failed to start 'rm' process");

        self.generate_cmd_summary(
            &format!("Deleting Repo {}", repo_name.to_string_lossy()),
            output,
        );

        self.app_screen = Screen::Summary;
    }

    fn checkout_new_worktree_repo(&mut self) {
        if let Some(ref i_state) = self.input_state {
            let repo_link = i_state.get_content_at(0);
            let chosen_repo_name = i_state.get_content_at(1);
            let repo_dir_name = Self::sanitise_git_dir_name(if chosen_repo_name.is_empty() {
                match Self::get_repo_name_from_git_link(&repo_link) {
                    Ok(name) => name,
                    Err(e) => {
                        self.summary_text = vec![e];
                        self.app_screen = Screen::Summary;
                        return;
                    }
                }
            } else {
                &chosen_repo_name
            });

            let Some(dir) = self.get_selected_pt_item() else {
                return;
            };

            let mkdir_output = Command::new("mkdir")
                .current_dir(&dir.path)
                .arg(&repo_dir_name)
                .output()
                .expect("Failed to start 'mkdir' process.");

            if !mkdir_output.status.success() {
                self.generate_cmd_summary(
                    &format!("Mkdir {} at {:?}", &repo_dir_name, dir.path),
                    mkdir_output,
                );
                return;
            }

            let mut repo_path = dir.path.clone();
            repo_path.push(&repo_dir_name);

            let clone_output = Command::new("git")
                .current_dir(&repo_path)
                .arg("clone")
                .arg("--bare")
                .arg(&repo_link)
                .arg(".bare")
                .output()
                .expect("Failed to start 'git clone' process");

            if !clone_output.status.success() {
                self.generate_cmd_summary("git clone", clone_output);
                return;
            }

            {
                let mut file_path = repo_path.clone();
                file_path.push(".git");
                let Ok(mut f) = File::create(&file_path) else {
                    self.summary_text =
                        vec![format!("Failed to create .git file at {:?}", &file_path)];
                    self.app_screen = Screen::Summary;
                    return;
                };
                if let Err(e) = f.write_all(b"gitdir: ./.bare") {
                    self.summary_text = vec![
                        format!("Failed to write to .git file at {:?}", &file_path),
                        format!("Error: {}", e),
                    ];
                    self.app_screen = Screen::Summary;
                    return;
                };
            }

            let config_output = Command::new("git")
                .current_dir(&repo_path)
                .arg("config")
                .arg("remote.origin.fetch")
                .arg("+refs/heads/*:refs/remotes/origin/*")
                .output()
                .expect("Failed to start 'git config' process");

            if !config_output.status.success() {
                self.generate_cmd_summary("git config", config_output);
                return;
            }

            let fetch_output = Command::new("git")
                .current_dir(&repo_path)
                .arg("fetch")
                .arg("origin")
                .output()
                .expect("failed to start 'git fetch' process");

            if !fetch_output.status.success() {
                self.generate_cmd_summary("git config", config_output);
                return;
            }

            self.summary_text = vec![format!(
                "Checked out new repo with name {} {:?}",
                repo_dir_name, &repo_path
            )];

            self.app_screen = Screen::Summary;
        }
    }

    fn checkout_new_non_worktree_repo(&mut self) {
        if let Some(ref i_state) = self.input_state {
            let repo_link = i_state.get_content_at(0);
            let chosen_repo_name = &i_state.get_content_at(1);
            let repo_name = Self::sanitise_git_dir_name(if chosen_repo_name.is_empty() {
                match Self::get_repo_name_from_git_link(&repo_link) {
                    Ok(name) => name,
                    Err(e) => {
                        self.summary_text = vec![e];
                        self.app_screen = Screen::Summary;
                        return;
                    }
                }
            } else {
                &chosen_repo_name
            });

            let Some(repo_dir) = self.get_selected_pt_item() else {
                return;
            };

            let output = Command::new("git")
                .current_dir(&repo_dir.path)
                .arg("clone")
                .arg(repo_link)
                .arg(&repo_name)
                .output()
                .expect("Failed to start 'git' process");

            self.generate_cmd_summary(
                &format!("Checking out new Non-Worktree Repo {}", repo_name),
                output,
            );

            self.app_screen = Screen::Summary;
        }
    }

    fn checkout_new_worktree(&mut self) {
        if let Some(ref i_state) = self.input_state {
            let branch_name = i_state.get_content_at(0);
            let chosen_dir_name = &i_state.get_content_at(1);
            let dir_name = Self::sanitise_git_dir_name(if chosen_dir_name.is_empty() {
                &branch_name
            } else {
                &chosen_dir_name
            });

            let Some(repo) = self.get_selected_pt_item() else {
                return;
            };

            let output = Command::new("git")
                .current_dir(&repo.path)
                .arg("worktree")
                .arg("add")
                .arg("-b")
                .arg(branch_name)
                .arg(&dir_name)
                .arg("--guess-remote")
                .output()
                .expect("Failed to start 'git' process");

            self.generate_cmd_summary(
                &format!(
                    "Checking out new Worktree {} in repo {}",
                    dir_name,
                    repo.path.file_name().unwrap().to_string_lossy()
                ),
                output,
            );

            self.app_screen = Screen::Summary;
        }
    }

    fn sanitise_git_dir_name(s: &str) -> String {
        s.replace("/", "_") // slashes will break the structure this project scans for dirs for.
            .replace(".", "_") // git doesnt like dots in dir paths
    }

    fn generate_cmd_summary(&mut self, desc: &str, output: Output) {
        if output.status.success() {
            self.summary_text = vec![format!("SUCCESS: {}", desc)];
        } else {
            self.summary_text = vec![
                format!("FAILURE: {}", desc),
                format!(
                    "STDOUT: {}",
                    from_utf8(&output.stdout).unwrap_or("couldnt read stdout as utf-8")
                ),
                format!(
                    "STDERR: {}",
                    from_utf8(&output.stderr).unwrap_or("couldnt read stderr as utf-8")
                ),
            ];
        }
    }

    fn get_repo_name_from_git_link(s: &str) -> Result<&str, String> {
        let Some((_, after_slash)) = s.rsplit_once('/') else {
            return Err(format!(
                "Could not interpret '{}' as git repository link.",
                s
            ));
        };
        let Some((default_name, _)) = after_slash.rsplit_once('.') else {
            return Err(format!(
                "Could not interpret '{}' as git repository link.",
                s
            ));
        };
        Ok(default_name)
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
