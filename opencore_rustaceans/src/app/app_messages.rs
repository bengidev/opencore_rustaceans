//! Shell-level messages routing to welcome or workspace features.

use crate::features::welcome::WelcomeMessage;
use crate::features::workspace::WorkspaceMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellMessage {
    Welcome(WelcomeMessage),
    Workspace(WorkspaceMessage),
    WindowCloseRequested,
}
