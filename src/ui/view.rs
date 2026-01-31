use crate::ui::state::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_app(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let items: Vec<ListItem> = app
        .view_items
        .iter()
        .map(|&idx| {
            let node = &app.nodes[idx];
            let indent = "  ".repeat(node.depth);
            let icon = if node.is_dir {
                if node.expanded {
                    "📂 "
                } else {
                    "📁 "
                }
            } else {
                "📄 "
            };
            let check = if node.selected { "[x] " } else { "[ ] " };

            let content = format!("{}{}{}{}", indent, check, icon, node.name);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Context Engine - Interactive Select "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    // We pass the mutable state here so Ratatui can update the scroll position
    frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

    let help_text = "Navigate: ↑/↓ | Toggle: Space | Expand/Collapse: Enter | Confirm: c | Quit: q";
    let help = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, chunks[1]);
}
