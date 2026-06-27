//! Composer scope — sandbox vs project folder context.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComposerScope {
    #[default]
    Sandbox,
    Folder,
}
