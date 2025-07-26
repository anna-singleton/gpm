#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Screen {
    #[default]
    Main,
    ScreenSwitchMenu,
    NonWorktreeRepoCreate,
    NonWorktreeRepoDelete,
    WorktreeRepoCreate,
    WorktreeRepoDelete,
    WorktreeCreate,
    WorktreeDelete,
    Summary,
}
