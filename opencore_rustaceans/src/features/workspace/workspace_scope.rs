//! Composer scope — project folder context.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComposerScope {
    #[default]
    Folder,
}
