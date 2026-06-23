//! Command palette model and fuzzy filtering.

use super::welcome_model::WelcomeItemId;

/// One selectable command in the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteCommand {
    pub id: WelcomeItemId,
    pub label: String,
    pub detail: Option<String>,
}

/// Build palette commands from get-started actions plus recent project labels.
pub fn palette_commands(recent_labels: &[String]) -> Vec<PaletteCommand> {
    let mut commands = vec![
        PaletteCommand {
            id: WelcomeItemId::NewFile,
            label: String::from("New File"),
            detail: Some(String::from("Create a new file")),
        },
        PaletteCommand {
            id: WelcomeItemId::OpenProject,
            label: String::from("Open Project"),
            detail: Some(String::from("Open a folder as a project")),
        },
        PaletteCommand {
            id: WelcomeItemId::CloneRepository,
            label: String::from("Clone Repository"),
            detail: Some(String::from("Clone a git repository")),
        },
        PaletteCommand {
            id: WelcomeItemId::OpenCommandPalette,
            label: String::from("Open Command Palette"),
            detail: Some(String::from("Show this palette")),
        },
    ];

    for (index, label) in recent_labels.iter().enumerate() {
        commands.push(PaletteCommand {
            id: WelcomeItemId::RecentProject(index),
            label: label.clone(),
            detail: Some(String::from("Open recent project")),
        });
    }

    commands
}

/// Filter commands by case-insensitive substring match on label and detail.
pub fn filter_commands(commands: &[PaletteCommand], query: &str) -> Vec<PaletteCommand> {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return commands.to_vec();
    }

    commands
        .iter()
        .filter(|command| {
            command.label.to_ascii_lowercase().contains(&needle)
                || command
                    .detail
                    .as_ref()
                    .is_some_and(|detail| detail.to_ascii_lowercase().contains(&needle))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_includes_recent_projects() {
        let commands = palette_commands(&[String::from("demo")]);
        assert_eq!(commands.len(), 5);
        assert_eq!(commands[4].id, WelcomeItemId::RecentProject(0));
    }

    #[test]
    fn filter_matches_label_substring() {
        let commands = palette_commands(&[]);
        let filtered = filter_commands(&commands, "clone");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, WelcomeItemId::CloneRepository);
    }

    #[test]
    fn empty_query_returns_all_commands() {
        let commands = palette_commands(&[]);
        assert_eq!(filter_commands(&commands, "").len(), commands.len());
    }
}
