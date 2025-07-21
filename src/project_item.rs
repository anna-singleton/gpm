use std::path::PathBuf;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ProjectItem {
    pub path: PathBuf,
    pub project_type: ProjectItemType,
    pub dirty: bool,
}

impl ProjectItem {
    pub fn new(path: PathBuf, project_type: ProjectItemType) -> Self {
        Self {
            path,
            project_type,
            dirty: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectItemType {
    Project,
    ProjectWorktree,
    ProjectDirectory,
}
