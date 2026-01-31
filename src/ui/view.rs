use crate::ui::state::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_app(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area()); // FIX: Use area() instead of size()

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

    frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

    let fmt_str = format!("{:?}", app.config.output_format);
    let clip_str = if app.config.to_clipboard {
        "[ON]"
    } else {
        "[OFF]"
    };
    let min_str = if app.config.minify { "[ON]" } else { "[OFF]" };

    let out_str = match &app.config.output_path {
        Some(p) => format!("File ({})", p.display()),
        None => "Natural (Stdout)".to_string(),
    };

    let status_text = format!(
        " Fmt(f): {} | Dest(o): {} | Clip(c): {} | Min(m): {} | Confirm: Enter ",
        fmt_str, out_str, clip_str, min_str
    );

    let help = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title(" Controls "))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(help, chunks[1]);
}