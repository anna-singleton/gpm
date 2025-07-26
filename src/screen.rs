#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Screen {
    #[default]
    Main,
    ScreenSwitchMenu,
    RepoCreate,
    RepoDelete,
    WorktreeCreate,
    WorktreeDelete,
    Summary,
}
