//! Welcome screen content model.
//!
//! Composite of sections and actionable rows. Static catalog for now;
//! a future Strategy adapter can swap in persisted recent projects.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeSectionId {
    GetStarted,
    RecentProjects,
}

/// One actionable row in a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WelcomeItem {
    pub id: WelcomeItemId,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub icon: WelcomeIcon,
}

/// A titled group of welcome rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WelcomeSection {
    pub id: WelcomeSectionId,
    pub title: &'static str,
    pub items: &'static [WelcomeItem],
}

/// Root content tree for the welcome screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WelcomeScreen {
    pub headline: &'static str,
    pub subtitle: &'static str,
    pub sections: &'static [WelcomeSection],
}

const GET_STARTED_ITEMS: [WelcomeItem; 4] = [
    WelcomeItem {
        id: WelcomeItemId::NewFile,
        label: "New File",
        shortcut: Some("⌘ N"),
        icon: WelcomeIcon::Plus,
    },
    WelcomeItem {
        id: WelcomeItemId::OpenProject,
        label: "Open Project",
        shortcut: Some("⌘ O"),
        icon: WelcomeIcon::Folder,
    },
    WelcomeItem {
        id: WelcomeItemId::CloneRepository,
        label: "Clone Repository",
        shortcut: None,
        icon: WelcomeIcon::CloudDownload,
    },
    WelcomeItem {
        id: WelcomeItemId::OpenCommandPalette,
        label: "Open Command Palette",
        shortcut: Some("⇧ ⇧"),
        icon: WelcomeIcon::CommandPalette,
    },
];

const RECENT_PROJECT_ITEMS: [WelcomeItem; 5] = [
    WelcomeItem {
        id: WelcomeItemId::RecentProject(0),
        label: "opencore_rustaceans",
        shortcut: Some("⌘ 1"),
        icon: WelcomeIcon::RecentFolder,
    },
    WelcomeItem {
        id: WelcomeItemId::RecentProject(1),
        label: "workspace-alpha",
        shortcut: Some("⌘ 2"),
        icon: WelcomeIcon::RecentFolder,
    },
    WelcomeItem {
        id: WelcomeItemId::RecentProject(2),
        label: "workspace-beta",
        shortcut: Some("⌘ 3"),
        icon: WelcomeIcon::RecentFolder,
    },
    WelcomeItem {
        id: WelcomeItemId::RecentProject(3),
        label: "playground",
        shortcut: Some("⌘ 4"),
        icon: WelcomeIcon::RecentFolder,
    },
    WelcomeItem {
        id: WelcomeItemId::RecentProject(4),
        label: "experiments",
        shortcut: Some("⌘ 5"),
        icon: WelcomeIcon::RecentFolder,
    },
];

const SECTIONS: [WelcomeSection; 2] = [
    WelcomeSection {
        id: WelcomeSectionId::GetStarted,
        title: "GET STARTED",
        items: &GET_STARTED_ITEMS,
    },
    WelcomeSection {
        id: WelcomeSectionId::RecentProjects,
        title: "RECENT PROJECTS",
        items: &RECENT_PROJECT_ITEMS,
    },
];

/// Default welcome catalog shown on launch.
pub fn default_screen() -> WelcomeScreen {
    WelcomeScreen {
        headline: "Welcome back to OpenCore",
        subtitle: "The workspace for what's next",
        sections: &SECTIONS,
    }
}

/// Flat item list across all sections (stable index order).
pub fn all_items(screen: &WelcomeScreen) -> Vec<WelcomeItem> {
    screen
        .sections
        .iter()
        .flat_map(|section| section.items.iter().copied())
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
    fn default_screen_has_two_sections_in_order() {
        let screen = default_screen();
        assert_eq!(screen.sections.len(), 2);
        assert_eq!(screen.sections[0].id, WelcomeSectionId::GetStarted);
        assert_eq!(screen.sections[1].id, WelcomeSectionId::RecentProjects);
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
    fn recent_projects_section_has_five_rows_with_shortcuts() {
        let screen = default_screen();
        let recent = &screen.sections[1];
        assert_eq!(recent.items.len(), 5);
        assert!(recent.items.iter().all(|item| item.shortcut.is_some()));
    }

    #[test]
    fn all_items_flattens_sections_in_order() {
        let screen = default_screen();
        let items = all_items(&screen);
        assert_eq!(items.len(), 9);
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
