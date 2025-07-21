use std::{fs::DirEntry, path::PathBuf};

use directories::UserDirs;
use serde::{Deserialize, Serialize};
use tui_tree_widget::{Tree, TreeItem};

use crate::project_item::{ProjectItem, ProjectItemType};

#[derive(Serialize, Deserialize)]
pub struct Config {
    project_directories: Vec<String>,
    standalone_projects: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_directories: vec!["~/proj".to_string()],
            standalone_projects: vec![],
        }
    }
}

impl Config {
    pub fn to_forest(&self) -> Vec<TreeItem<ProjectItem>> {
        let mut forest = vec![];

        let home_dir = UserDirs::new().unwrap().home_dir().to_path_buf();

        for proj in self.standalone_projects.iter() {
            let path: PathBuf = if proj.starts_with("~/") {
                home_dir.join(PathBuf::from(proj.strip_prefix("~/").unwrap()))
            } else {
                home_dir.join(PathBuf::from(proj))
            };
            let tree_item = ProjectItem::new(path.clone(), ProjectItemType::Project);
            let name: String = path.file_name().unwrap().to_str().unwrap().to_owned();
            forest.push(TreeItem::new_leaf(tree_item, name));
        }

        for project_dir in self.project_directories.iter() {
            let path: PathBuf = if project_dir.starts_with("~/") {
                home_dir.join(PathBuf::from(project_dir.strip_prefix("~/").unwrap()))
            } else {
                home_dir.join(PathBuf::from(project_dir))
            };
            let Ok(contents) = path.read_dir() else {
                eprintln!(
                    "{} was set as a project directory but is not a directory. skipping.",
                    path.to_string_lossy()
                );
                continue;
            };

            let ct: Vec<DirEntry> = contents.into_iter().filter_map(|d| d.ok()).collect();

            let mut children: Vec<TreeItem<ProjectItem>> = vec![];

            for subdir in ct.iter() {
                if !subdir.path().is_dir() {
                    continue;
                }

                let Ok(subdir_contents) = subdir.path().read_dir() else {
                    eprintln!(
                        "{} was found as a subdir but is not a directory. skipping.",
                        path.to_string_lossy()
                    );
                    continue;
                };

                let subdir_ct: Vec<DirEntry> =
                    subdir_contents.into_iter().filter_map(|d| d.ok()).collect();

                if !subdir_ct.iter().any(|d| d.file_name() == ".bare") {
                    // this is a project, not a project home.
                    let name = subdir
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();

                    children.push(TreeItem::new_leaf(
                        ProjectItem::new(subdir.path(), ProjectItemType::Project),
                        name,
                    ));
                    continue;
                }

                let mut sub_children: Vec<TreeItem<ProjectItem>> = vec![];

                for proj in subdir_ct.iter() {
                    let proj_path = proj.path();
                    let name = proj
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                    if name == ".bare" || name == ".git" {
                        continue;
                    }
                    sub_children.push(TreeItem::new_leaf(
                        ProjectItem::new(proj_path, ProjectItemType::Project),
                        name
                    ));
                }
                sub_children.sort_by_key(|c| c.identifier().path.file_name().unwrap().to_string_lossy().into_owned());

                children.push(
                    TreeItem::new(
                        ProjectItem::new(subdir.path(), ProjectItemType::ProjectWorktree),
                        subdir
                            .path()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        sub_children,
                    )
                    .unwrap(),
                );
            }

            children.sort_by_key(|c| c.identifier().path.file_name().unwrap().to_string_lossy().into_owned());

            let project_dir_tree_item = TreeItem::new(
                ProjectItem::new(path.clone(), ProjectItemType::ProjectDirectory),
                path.file_name().unwrap().to_string_lossy().into_owned(),
                children
            ).unwrap();

            forest.push(project_dir_tree_item);
        }
        return forest;
    }
}
