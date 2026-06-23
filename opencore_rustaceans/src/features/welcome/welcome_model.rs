//! Welcome screen content model.
//!
//! Composite of sections and actionable rows. Recent projects are supplied
//! at build time; the section is omitted when history is empty.

use std::path::PathBuf;

use super::welcome_history::project_label;

/// Stable identifier for a welcome row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WelcomeItemId {
    NewFile,
    OpenProject,
    CloneRepository,
    OpenCommandPalette,
    RecentProject(usize),
}

/// Visual icon kind for a welcome row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeIcon {
    Plus,
    Folder,
    CloudDownload,
    CommandPalette,
    RecentFolder,
}

/// Section grouping on the welcome screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WelcomeSectionId {
    GetStarted,
    RecentProjects,
}

/// One actionable row in a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WelcomeItem {
    pub id: WelcomeItemId,
    pub label: String,
    pub shortcut: Option<String>,
    pub icon: WelcomeIcon,
}

/// A titled group of welcome rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WelcomeSection {
    pub id: WelcomeSectionId,
    pub title: &'static str,
    pub items: Vec<WelcomeItem>,
}

/// Root content tree for the welcome screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WelcomeScreen {
    pub headline: &'static str,
    pub subtitle: &'static str,
    pub sections: Vec<WelcomeSection>,
}

fn get_started_section() -> WelcomeSection {
    WelcomeSection {
        id: WelcomeSectionId::GetStarted,
        title: "GET STARTED",
        items: vec![
            WelcomeItem {
                id: WelcomeItemId::NewFile,
                label: String::from("New File"),
                shortcut: Some(String::from("⌘ N")),
                icon: WelcomeIcon::Plus,
            },
            WelcomeItem {
                id: WelcomeItemId::OpenProject,
                label: String::from("Open Project"),
                shortcut: Some(String::from("⌘ O")),
                icon: WelcomeIcon::Folder,
            },
            WelcomeItem {
                id: WelcomeItemId::CloneRepository,
                label: String::from("Clone Repository"),
                shortcut: None,
                icon: WelcomeIcon::CloudDownload,
            },
            WelcomeItem {
                id: WelcomeItemId::OpenCommandPalette,
                label: String::from("Open Command Palette"),
                shortcut: Some(String::from("⇧ ⇧")),
                icon: WelcomeIcon::CommandPalette,
            },
        ],
    }
}

fn recent_projects_section(names: &[String]) -> WelcomeSection {
    let items = names
        .iter()
        .enumerate()
        .map(|(index, name)| WelcomeItem {
            id: WelcomeItemId::RecentProject(index),
            label: name.clone(),
            shortcut: Some(format!("⌘ {}", index + 1)),
            icon: WelcomeIcon::RecentFolder,
        })
        .collect();

    WelcomeSection {
        id: WelcomeSectionId::RecentProjects,
        title: "RECENT PROJECTS",
        items,
    }
}

/// Build the welcome catalog from recently accessed project paths.
///
/// Omits the recent-projects section when `recent_paths` is empty.
pub fn build_screen(recent_paths: &[PathBuf]) -> WelcomeScreen {
    let mut sections = vec![get_started_section()];
    if !recent_paths.is_empty() {
        let names: Vec<String> = recent_paths.iter().map(|path| project_label(path)).collect();
        sections.push(recent_projects_section(&names));
    }

    WelcomeScreen {
        headline: "Welcome back to OpenCore",
        subtitle: "The workspace for what's next",
        sections,
    }
}

/// Default welcome catalog with no recent-project history.
pub fn default_screen() -> WelcomeScreen {
    build_screen(&[])
}

/// Flat item list across all sections (stable index order).
pub fn all_items(screen: &WelcomeScreen) -> Vec<&WelcomeItem> {
    screen
        .sections
        .iter()
        .flat_map(|section| section.items.iter())
        .collect()
}

/// Resolve a flat item index to its stable id.
pub fn item_id_at(screen: &WelcomeScreen, index: usize) -> Option<WelcomeItemId> {
    all_items(screen).get(index).map(|item| item.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_screen_shows_only_get_started_when_history_empty() {
        let screen = default_screen();
        assert_eq!(screen.sections.len(), 1);
        assert_eq!(screen.sections[0].id, WelcomeSectionId::GetStarted);
    }

    #[test]
    fn get_started_section_has_four_action_rows() {
        let screen = default_screen();
        assert_eq!(screen.sections[0].items.len(), 4);
        assert_eq!(
            screen.sections[0].items[0].id,
            WelcomeItemId::NewFile
        );
    }

    #[test]
    fn build_screen_includes_recent_section_when_history_present() {
        let screen = build_screen(&[
            PathBuf::from("/tmp/opencore_rustaceans"),
            PathBuf::from("/tmp/playground"),
        ]);
        assert_eq!(screen.sections.len(), 2);
        assert_eq!(screen.sections[1].id, WelcomeSectionId::RecentProjects);
        assert_eq!(screen.sections[1].items.len(), 2);
        assert_eq!(screen.sections[1].items[0].label, "opencore_rustaceans");
        assert_eq!(
            screen.sections[1].items[0].shortcut.as_deref(),
            Some("⌘ 1")
        );
    }

    #[test]
    fn all_items_flattens_sections_in_order() {
        let screen = build_screen(&[PathBuf::from("/tmp/demo")]);
        let items = all_items(&screen);
        assert_eq!(items.len(), 5);
        assert_eq!(items[0].id, WelcomeItemId::NewFile);
        assert_eq!(items[3].id, WelcomeItemId::OpenCommandPalette);
        assert_eq!(items[4].id, WelcomeItemId::RecentProject(0));
    }

    #[test]
    fn item_id_at_resolves_flat_index() {
        let screen = default_screen();
        assert_eq!(
            item_id_at(&screen, 2),
            Some(WelcomeItemId::CloneRepository)
        );
        assert_eq!(item_id_at(&screen, 99), None);
    }
}
