//! Active overlay on the welcome screen.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WelcomeOverlay {
    #[default]
    None,
    CommandPalette,
    CloneRepository,
}
