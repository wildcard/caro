// ============================================================================
// SIMPLE COMPONENT TEMPLATE
// ============================================================================
// This is a template for creating new showcase components. Copy this file
// to src/tui/components/ and customize it for your component.
//
// QUICK START:
// 1. Copy this file: cp docs/templates/simple_component_template.rs \
//                       src/tui/components/my_component.rs
// 2. Replace all instances of "SimpleComponentTemplate" with "MyComponent"
// 3. Update the metadata (name, description, etc.)
// 4. Customize the stories (the render functions)
// 5. Add to mod.rs and tui_showcase.rs (see instructions below)
// 6. Run: cargo run --bin tui-showcase
//
// HELP: See GETTING_STARTED.md for detailed instructions!
// ============================================================================

// ----------------------------------------------------------------------------
// IMPORTS
// ----------------------------------------------------------------------------
// These are the essential imports you'll need for most components.
// You can add more as needed!

// Ratatui layout module - for positioning widgets
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    // Ratatui style module - for colors and text styling
    style::{Color, Modifier, Style},
    // Ratatui text module - for creating text content
    text::{Line, Span, Text},
    // Ratatui widgets - building blocks for your UI
    widgets::{Block, Borders, Paragraph, Widget},
    // Frame is the drawing canvas
    Frame,
};

// Standard I/O - required by the ShowcaseComponent trait
use std::io;

// Our showcase framework - this defines the component interface
use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};

// ----------------------------------------------------------------------------
// COMPONENT STRUCT
// ----------------------------------------------------------------------------
// This struct represents your component. For simple components, it can be
// empty. For more complex components, you can add fields to store state.
//
// EXAMPLES:
//   pub struct SimpleComponentTemplate;  // No state
//   pub struct CounterComponent { count: usize }  // With state
//   pub struct FormComponent { fields: Vec<String> }  // Complex state

pub struct SimpleComponentTemplate;

// If your component needs state, you might initialize it like this:
// impl SimpleComponentTemplate {
//     pub fn new() -> Self {
//         SimpleComponentTemplate {
//             // Initialize fields here
//         }
//     }
// }

// ----------------------------------------------------------------------------
// SHOWCASE COMPONENT IMPLEMENTATION
// ----------------------------------------------------------------------------
// This is where you define how your component appears in the showcase.

impl ShowcaseComponent for SimpleComponentTemplate {
    // ------------------------------------------------------------------------
    // METADATA
    // ------------------------------------------------------------------------
    // This function returns information about your component that appears
    // in the showcase browser.
    //
    // TIP: Choose a descriptive name and category!
    //
    // Common categories:
    // - "Display" - Components that show information
    // - "Input" - Components for user input
    // - "Feedback" - Progress, notifications, confirmations
    // - "Workflow" - Multi-step processes
    // - "Help" - Documentation, shortcuts, guides

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            // The name shown in the showcase browser
            name: "Simple Template".to_string(),

            // Brief description of what this component does
            description: "A template for creating new components".to_string(),

            // Category for organization (see list above)
            category: "Display".to_string(),

            // Semantic version
            version: "1.0.0".to_string(),
        }
    }

    // ------------------------------------------------------------------------
    // STORIES
    // ------------------------------------------------------------------------
    // Stories are different variations or states of your component.
    // Each story is like an example or demo of your component.
    //
    // EXAMPLES:
    // - A button might have: Normal, Hovered, Pressed, Disabled stories
    // - A status indicator might have: Success, Warning, Error stories
    // - A dialog might have: Question, Confirmation, Alert stories
    //
    // TIP: Create at least 2-3 stories showing different states!

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            // ----------------------------------------------------------------
            // STORY 1: Basic Example
            // ----------------------------------------------------------------
            // This story shows the simplest version of your component.

            ShowcaseStory {
                // Name of this story (shown in story list)
                name: "Basic Text".to_string(),

                // Description of what this story demonstrates
                description: "Simple text display with a border".to_string(),

                // The render function - this draws your component!
                // The |frame| syntax creates a closure (anonymous function).
                // Think of 'frame' as your drawing canvas.
                render: Box::new(|frame: &mut Frame| {
                    // Get the available drawing area
                    // This is a Rect (rectangle) with x, y, width, height
                    let area = frame.area();

                    // Create a text widget
                    // Paragraph is a widget that displays text
                    let text = Paragraph::new("Hello, World!")
                        // Center the text
                        .alignment(Alignment::Center)
                        // Add a border around it
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Simple Component"),
                        );

                    // Render the widget to the frame
                    // This is what actually draws it!
                    frame.render_widget(text, area);
                }),
            },
            // ----------------------------------------------------------------
            // STORY 2: With Colors
            // ----------------------------------------------------------------
            // This story shows the same component with styling.

            ShowcaseStory {
                name: "Colored Text".to_string(),
                description: "Text with colors and styling".to_string(),
                render: Box::new(|frame: &mut Frame| {
                    let area = frame.area();

                    // Create styled text using Span
                    // Span lets you apply styles to individual parts
                    let styled_text = Line::from(vec![
                        // Normal text
                        Span::raw("This is "),
                        // Green text
                        Span::styled(
                            "green",
                            Style::default().fg(Color::Green),
                        ),
                        Span::raw(" and this is "),
                        // Bold red text
                        Span::styled(
                            "bold red",
                            Style::default()
                                .fg(Color::Red)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("!"),
                    ]);

                    let paragraph = Paragraph::new(styled_text)
                        .alignment(Alignment::Center)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Styled")
                                .style(Style::default().fg(Color::Cyan)),
                        );

                    frame.render_widget(paragraph, area);
                }),
            },
            // ----------------------------------------------------------------
            // STORY 3: Multi-line with Layout
            // ----------------------------------------------------------------
            // This story shows how to create multiple sections.

            ShowcaseStory {
                name: "Multi-section".to_string(),
                description: "Multiple sections with layout".to_string(),
                render: Box::new(|frame: &mut Frame| {
                    let area = frame.area();

                    // Split the area into three sections
                    // Layout divides the space based on constraints
                    let chunks = Layout::default()
                        // Stack vertically (top to bottom)
                        .direction(Direction::Vertical)
                        // Define sizes for each section
                        .constraints([
                            // First section: exactly 3 rows
                            Constraint::Length(3),
                            // Second section: at least 5 rows
                            Constraint::Min(5),
                            // Third section: exactly 3 rows
                            Constraint::Length(3),
                        ])
                        // Apply to the available area
                        .split(area);

                    // Now we have 3 rectangles: chunks[0], chunks[1], chunks[2]

                    // Render header in first section
                    let header = Paragraph::new("Header Section")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(header, chunks[0]);

                    // Render body in second section
                    let body = Paragraph::new(vec![
                        Line::from("This is the main content area."),
                        Line::from("You can put multiple lines here."),
                        Line::from("It will expand to fill available space."),
                    ])
                    .block(Block::default().borders(Borders::ALL).title("Body"));
                    frame.render_widget(body, chunks[1]);

                    // Render footer in third section
                    let footer = Paragraph::new("Footer Section")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Gray))
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(footer, chunks[2]);
                }),
            },
        ]

        // TIP: You can add more stories here! Just follow the same pattern.
        // Each story should demonstrate a different aspect or state.
    }

    // ------------------------------------------------------------------------
    // OPTIONAL HOOKS
    // ------------------------------------------------------------------------
    // These methods are optional. Only implement them if you need them!

    // Called when user navigates to a story (optional)
    // fn init(&mut self) -> io::Result<()> {
    //     // Do setup here (start timers, load data, etc.)
    //     Ok(())
    // }

    // Called when user navigates away from a story (optional)
    // fn cleanup(&mut self) -> io::Result<()> {
    //     // Do cleanup here (stop timers, free resources, etc.)
    //     Ok(())
    // }

    // Handle keyboard input (optional - for interactive components)
    // fn handle_key_event(&mut self, event: crossterm::event::KeyEvent) -> io::Result<bool> {
    //     use crossterm::event::KeyCode;
    //
    //     match event.code {
    //         KeyCode::Up => {
    //             // Handle up arrow
    //             Ok(true)  // Return true if we handled it
    //         }
    //         KeyCode::Down => {
    //             // Handle down arrow
    //             Ok(true)
    //         }
    //         _ => Ok(false)  // Return false if we didn't handle it
    //     }
    // }
}

// ============================================================================
// REGISTRATION INSTRUCTIONS
// ============================================================================
// After creating your component, you need to register it in two places:
//
// 1. Add to src/tui/components/mod.rs:
//    ```rust
//    pub mod my_component;  // Replace with your component file name
//    pub use my_component::MyComponent;  // Replace with your component name
//    ```
//
// 2. Add to src/bin/tui_showcase.rs:
//    ```rust
//    // At the top with other imports:
//    use cmdai::tui::components::MyComponent;
//
//    // Inside ShowcaseBrowser::new(), in the registry section:
//    registry.register(Box::new(MyComponent));
//    ```
//
// Then run:
//    cargo run --bin tui-showcase
//
// Your component should appear in the showcase!
// ============================================================================

// ============================================================================
// COMMON PATTERNS AND EXAMPLES
// ============================================================================
//
// Here are some copy-paste examples for common tasks:
//
// 1. CENTER CONTENT:
//    .alignment(Alignment::Center)
//
// 2. CHANGE TEXT COLOR:
//    Span::styled("text", Style::default().fg(Color::Green))
//
// 3. BOLD TEXT:
//    Style::default().add_modifier(Modifier::BOLD)
//
// 4. BORDER WITH TITLE:
//    .block(Block::default().borders(Borders::ALL).title("My Title"))
//
// 5. MULTIPLE COLORS IN ONE LINE:
//    Line::from(vec![
//        Span::raw("Normal "),
//        Span::styled("Red", Style::default().fg(Color::Red)),
//    ])
//
// 6. SPLIT SCREEN VERTICALLY:
//    let chunks = Layout::default()
//        .direction(Direction::Vertical)
//        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
//        .split(area);
//
// 7. SPLIT SCREEN HORIZONTALLY:
//    let chunks = Layout::default()
//        .direction(Direction::Horizontal)
//        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
//        .split(area);
//
// 8. RESPONSIVE LAYOUT (adapts to terminal size):
//    let constraints = if area.width < 60 {
//        vec![Constraint::Percentage(100)]  // Single column on small screens
//    } else {
//        vec![Constraint::Percentage(50), Constraint::Percentage(50)]  // Two columns
//    };
//
// ============================================================================
// NEED HELP?
// ============================================================================
// - Read GETTING_STARTED.md for a beginner-friendly guide
// - Check ARCHITECTURE_GUIDE.md to understand how it works
// - See FAQ.md for answers to common questions
// - Look at existing components in src/tui/components/ for examples
// - Ask in GitHub issues with the 'question' label!
//
// Happy coding! 🚀
// ============================================================================
