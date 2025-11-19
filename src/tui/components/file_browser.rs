//! File Browser Component - Hierarchical file/directory tree display
//!
//! This component displays a hierarchical file tree with expandable folders,
//! file type icons, and selection highlighting. Perfect for demonstrating
//! tree structures and nested data visualization in terminal UIs.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Represents a file or directory in the tree
#[derive(Debug, Clone, PartialEq)]
pub struct FileNode {
    pub name: String,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub children: Vec<FileNode>,
    pub depth: usize,
    pub is_selected: bool,
}

impl FileNode {
    /// Creates a new file node
    pub fn file(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_directory: false,
            is_expanded: false,
            children: Vec::new(),
            depth: 0,
            is_selected: false,
        }
    }

    /// Creates a new directory node
    pub fn directory(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_directory: true,
            is_expanded: false,
            children: Vec::new(),
            depth: 0,
            is_selected: false,
        }
    }

    /// Adds a child node and sets its depth recursively
    pub fn with_child(mut self, mut child: FileNode) -> Self {
        child.set_depth(self.depth + 1);
        self.children.push(child);
        self
    }

    /// Marks this directory as expanded
    pub fn expanded(mut self) -> Self {
        self.is_expanded = true;
        self
    }

    /// Marks this node as selected
    pub fn selected(mut self) -> Self {
        self.is_selected = true;
        self
    }

    /// Recursively sets depth for all children
    fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
        for child in &mut self.children {
            child.set_depth(depth + 1);
        }
    }
}

/// File Browser Component
pub struct FileBrowserComponent;

impl FileBrowserComponent {
    /// Renders a file tree from a list of nodes
    fn render_file_tree(nodes: &[FileNode], area: Rect, frame: &mut Frame, title: &str) {
        let items: Vec<ListItem> = nodes
            .iter()
            .flat_map(|node| Self::flatten_tree(node))
            .map(|node| Self::create_list_item(&node))
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(list, area);
    }

    /// Flattens a tree structure into a list of visible nodes
    fn flatten_tree(node: &FileNode) -> Vec<FileNode> {
        let mut result = vec![node.clone()];

        if node.is_directory && node.is_expanded {
            for child in &node.children {
                result.extend(Self::flatten_tree(child));
            }
        }

        result
    }

    /// Creates a ListItem from a FileNode with proper styling
    fn create_list_item(node: &FileNode) -> ListItem<'static> {
        let indent = "  ".repeat(node.depth);
        let icon = Self::get_icon(node);
        let folder_indicator = if node.is_directory {
            if node.is_expanded {
                "▼ "
            } else {
                "▶ "
            }
        } else {
            "  "
        };

        let style = if node.is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if node.is_directory {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let line = Line::from(vec![
            Span::raw(indent),
            Span::raw(folder_indicator),
            Span::raw(icon),
            Span::raw(" "),
            Span::styled(node.name.clone(), style),
        ]);

        ListItem::new(line)
    }

    /// Gets the appropriate icon for a file based on its extension
    fn get_icon(node: &FileNode) -> &'static str {
        if node.is_directory {
            return "📁";
        }

        // Get file extension
        let parts: Vec<&str> = node.name.split('.').collect();
        if parts.len() < 2 {
            return "📄";
        }

        match parts.last().unwrap().to_lowercase().as_str() {
            "rs" => "🦀",
            "py" => "🐍",
            "js" | "ts" => "📜",
            "json" | "yaml" | "yml" | "toml" => "⚙️",
            "md" | "txt" => "📝",
            "pdf" => "📕",
            "zip" | "tar" | "gz" => "📦",
            "jpg" | "png" | "gif" | "svg" => "🖼️",
            _ => "📄",
        }
    }

    /// Renders an empty directory message
    fn render_empty_directory(frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Empty Directory",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from(vec![Span::raw("No files or folders to display")]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("File Browser")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

impl ShowcaseComponent for FileBrowserComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "FileBrowser",
            "Hierarchical file/directory tree with expandable folders and file type icons",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            // Story 1: Simple Tree
            ShowcaseStory::new(
                "Simple Tree",
                "Basic file tree with 3-4 files and folders, some expanded",
                |frame, area| {
                    let tree = vec![
                        FileNode::directory("src")
                            .expanded()
                            .with_child(FileNode::file("main.rs"))
                            .with_child(FileNode::file("lib.rs")),
                        FileNode::directory("tests")
                            .with_child(FileNode::file("integration_test.rs")),
                        FileNode::file("Cargo.toml"),
                        FileNode::file("README.md"),
                    ];

                    FileBrowserComponent::render_file_tree(
                        &tree,
                        area,
                        frame,
                        "File Browser - Simple Tree",
                    );
                },
            ),
            // Story 2: Deep Nesting
            ShowcaseStory::new(
                "Deep Nesting",
                "Multiple levels of nesting (5+ levels deep)",
                |frame, area| {
                    let tree = vec![FileNode::directory("project").expanded().with_child(
                        FileNode::directory("src").expanded().with_child(
                            FileNode::directory("components").expanded().with_child(
                                FileNode::directory("ui").expanded().with_child(
                                    FileNode::directory("buttons")
                                        .expanded()
                                        .with_child(FileNode::file("PrimaryButton.rs"))
                                        .with_child(FileNode::file("SecondaryButton.rs")),
                                ),
                            ),
                        ),
                    )];

                    FileBrowserComponent::render_file_tree(
                        &tree,
                        area,
                        frame,
                        "File Browser - Deep Nesting",
                    );
                },
            ),
            // Story 3: Large Directory
            ShowcaseStory::new(
                "Large Directory",
                "Directory with many items (20+ files)",
                |frame, area| {
                    let mut root = FileNode::directory("logs").expanded();

                    // Add 20+ log files
                    for i in 1..=22 {
                        let filename = format!("app-{:04}.log", i);
                        root = root.with_child(FileNode::file(&filename));
                    }

                    let tree = vec![root];
                    FileBrowserComponent::render_file_tree(
                        &tree,
                        area,
                        frame,
                        "File Browser - Large Directory",
                    );
                },
            ),
            // Story 4: With Icons
            ShowcaseStory::new(
                "With Icons",
                "File type icons using Unicode characters",
                |frame, area| {
                    let tree = vec![FileNode::directory("project")
                        .expanded()
                        .with_child(FileNode::file("main.rs"))
                        .with_child(FileNode::file("script.py"))
                        .with_child(FileNode::file("config.json"))
                        .with_child(FileNode::file("README.md"))
                        .with_child(FileNode::file("logo.png"))
                        .with_child(FileNode::file("manual.pdf"))
                        .with_child(FileNode::file("archive.zip"))
                        .with_child(FileNode::file("index.js"))];

                    FileBrowserComponent::render_file_tree(
                        &tree,
                        area,
                        frame,
                        "File Browser - With Icons",
                    );
                },
            ),
            // Story 5: Selected Item
            ShowcaseStory::new(
                "Selected Item",
                "Show highlighted/selected file with cyan background",
                |frame, area| {
                    let tree = vec![
                        FileNode::directory("src")
                            .expanded()
                            .with_child(FileNode::file("main.rs").selected())
                            .with_child(FileNode::file("lib.rs"))
                            .with_child(FileNode::file("utils.rs")),
                        FileNode::file("Cargo.toml"),
                    ];

                    FileBrowserComponent::render_file_tree(
                        &tree,
                        area,
                        frame,
                        "File Browser - Selected Item",
                    );
                },
            ),
            // Story 6: Empty Directory
            ShowcaseStory::new(
                "Empty Directory",
                "Edge case showing an empty directory with no files",
                |frame, area| {
                    FileBrowserComponent::render_empty_directory(frame, area);
                },
            ),
            // Story 7: Search Results
            ShowcaseStory::new(
                "Search Results",
                "Filtered view showing only Rust files",
                |frame, area| {
                    let tree = vec![FileNode::directory("Search Results: *.rs")
                        .expanded()
                        .with_child(FileNode::file("src/main.rs"))
                        .with_child(FileNode::file("src/lib.rs"))
                        .with_child(FileNode::file("src/utils.rs"))
                        .with_child(FileNode::file("tests/integration.rs"))
                        .with_child(FileNode::file("tests/unit.rs"))];

                    FileBrowserComponent::render_file_tree(
                        &tree,
                        area,
                        frame,
                        "File Browser - Search Results",
                    );
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_node_creation() {
        let file = FileNode::file("test.txt");
        assert_eq!(file.name, "test.txt");
        assert!(!file.is_directory);
        assert!(!file.is_expanded);
        assert_eq!(file.children.len(), 0);
        assert_eq!(file.depth, 0);

        let dir = FileNode::directory("src");
        assert_eq!(dir.name, "src");
        assert!(dir.is_directory);
        assert!(!dir.is_expanded);
    }

    #[test]
    fn test_file_node_with_child() {
        let file = FileNode::file("main.rs");
        let dir = FileNode::directory("src").with_child(file);

        assert_eq!(dir.children.len(), 1);
        assert_eq!(dir.children[0].name, "main.rs");
        assert_eq!(dir.children[0].depth, 1);
    }

    #[test]
    fn test_file_node_expanded() {
        let dir = FileNode::directory("src").expanded();
        assert!(dir.is_expanded);
    }

    #[test]
    fn test_file_node_selected() {
        let file = FileNode::file("test.rs").selected();
        assert!(file.is_selected);
    }

    #[test]
    fn test_flatten_tree_simple() {
        let tree = FileNode::directory("root")
            .with_child(FileNode::file("a.txt"))
            .with_child(FileNode::file("b.txt"));

        let flattened = FileBrowserComponent::flatten_tree(&tree);
        assert_eq!(flattened.len(), 1); // Only root, children not visible when collapsed
    }

    #[test]
    fn test_flatten_tree_expanded() {
        let tree = FileNode::directory("root")
            .expanded()
            .with_child(FileNode::file("a.txt"))
            .with_child(FileNode::file("b.txt"));

        let flattened = FileBrowserComponent::flatten_tree(&tree);
        assert_eq!(flattened.len(), 3); // Root + 2 children
    }

    #[test]
    fn test_flatten_tree_nested_expanded() {
        let tree = FileNode::directory("root").expanded().with_child(
            FileNode::directory("sub")
                .expanded()
                .with_child(FileNode::file("nested.txt")),
        );

        let flattened = FileBrowserComponent::flatten_tree(&tree);
        assert_eq!(flattened.len(), 3); // Root + sub + nested.txt
    }

    #[test]
    fn test_get_icon_directory() {
        let dir = FileNode::directory("src");
        assert_eq!(FileBrowserComponent::get_icon(&dir), "📁");
    }

    #[test]
    fn test_get_icon_rust_file() {
        let file = FileNode::file("main.rs");
        assert_eq!(FileBrowserComponent::get_icon(&file), "🦀");
    }

    #[test]
    fn test_get_icon_python_file() {
        let file = FileNode::file("script.py");
        assert_eq!(FileBrowserComponent::get_icon(&file), "🐍");
    }

    #[test]
    fn test_get_icon_json_file() {
        let file = FileNode::file("config.json");
        assert_eq!(FileBrowserComponent::get_icon(&file), "⚙️");
    }

    #[test]
    fn test_get_icon_markdown_file() {
        let file = FileNode::file("README.md");
        assert_eq!(FileBrowserComponent::get_icon(&file), "📝");
    }

    #[test]
    fn test_get_icon_unknown_file() {
        let file = FileNode::file("unknown.xyz");
        assert_eq!(FileBrowserComponent::get_icon(&file), "📄");
    }

    #[test]
    fn test_component_metadata() {
        let component = FileBrowserComponent;
        let metadata = component.metadata();

        assert_eq!(metadata.name, "FileBrowser");
        assert_eq!(metadata.category, "Display");
        assert_eq!(metadata.version, "1.0.0");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_component_has_seven_stories() {
        let component = FileBrowserComponent;
        let stories = component.stories();

        assert_eq!(stories.len(), 7);
    }

    #[test]
    fn test_story_names() {
        let component = FileBrowserComponent;
        let stories = component.stories();

        assert_eq!(stories[0].name, "Simple Tree");
        assert_eq!(stories[1].name, "Deep Nesting");
        assert_eq!(stories[2].name, "Large Directory");
        assert_eq!(stories[3].name, "With Icons");
        assert_eq!(stories[4].name, "Selected Item");
        assert_eq!(stories[5].name, "Empty Directory");
        assert_eq!(stories[6].name, "Search Results");
    }

    #[test]
    fn test_depth_calculation() {
        let tree = FileNode::directory("root").with_child(
            FileNode::directory("level1")
                .with_child(FileNode::directory("level2").with_child(FileNode::file("deep.txt"))),
        );

        assert_eq!(tree.depth, 0);
        assert_eq!(tree.children[0].depth, 1);
        assert_eq!(tree.children[0].children[0].depth, 2);
        assert_eq!(tree.children[0].children[0].children[0].depth, 3);
    }
}
