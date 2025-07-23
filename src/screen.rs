#[derive(Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Main,
    CheckoutNewWorktree,
    BranchCreate,
    WorktreeDelete,
    BranchDelete,
    ProjectMenu,
    ProjectWorktreeMenu,
    ProjectDirectoryMenu,
}
