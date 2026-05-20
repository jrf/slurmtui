use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::App;
use crate::colors;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let title = format!(" Partitions ({}) ", app.partitions.len());

    let header = Row::new(vec![
        Cell::from("Partition"),
        Cell::from("Avail"),
        Cell::from("TimeLimit"),
        Cell::from("Nodes"),
        Cell::from("State"),
        Cell::from("CPUs"),
        Cell::from("Mem(GB)"),
        Cell::from("GRES"),
        Cell::from("NodeList"),
    ])
    .style(
        Style::default()
            .fg(colors::BLUE)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .partitions
        .iter()
        .map(|p| {
            let state_color = match p.state.as_str() {
                "idle" => colors::GREEN,
                "mix" => colors::YELLOW,
                "alloc" | "allocated" => colors::BLUE1,
                "down" | "down*" => colors::RED,
                "drain" | "drng" => colors::COMMENT,
                _ => colors::FG,
            };

            let mem_gb = if p.memory_mb > 0 {
                format!("{}", p.memory_mb / 1024)
            } else {
                "-".to_string()
            };

            let gres_display = if p.gres == "(null)" {
                "-".to_string()
            } else {
                p.gres.clone()
            };

            let partition_style = if p.partition.ends_with('*') {
                Style::default().fg(colors::FG).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::FG)
            };

            Row::new(vec![
                Cell::from(Span::styled(p.partition.as_str(), partition_style)),
                Cell::from(Span::styled(p.avail.as_str(), Style::default().fg(colors::FG_DARK))),
                Cell::from(Span::styled(p.time_limit.as_str(), Style::default().fg(colors::DARK5))),
                Cell::from(Span::styled(p.nodes.to_string(), Style::default().fg(colors::MAGENTA))),
                Cell::from(Span::styled(p.state.as_str(), Style::default().fg(state_color))),
                Cell::from(Span::styled(p.cpus_per_node.to_string(), Style::default().fg(colors::MAGENTA))),
                Cell::from(Span::styled(mem_gb, Style::default().fg(colors::MAGENTA))),
                Cell::from(Span::styled(gres_display, Style::default().fg(colors::CYAN))),
                Cell::from(Span::styled(p.nodelist.as_str(), Style::default().fg(colors::FG_DARK))),
            ])
        })
        .collect();

    if rows.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled("No partition data", Style::default().fg(colors::COMMENT))).centered())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::FG_GUTTER))
                    .title(Span::styled(title, Style::default().fg(colors::BLUE))),
            );
        frame.render_widget(msg, area);
    } else {
        let widths = [
            Constraint::Length(14),
            Constraint::Length(6),
            Constraint::Length(14),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(22),
            Constraint::Fill(1),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::FG_GUTTER))
                    .title(Span::styled(title, Style::default().fg(colors::BLUE))),
            )
            .row_highlight_style(
                Style::default()
                    .bg(colors::BG_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(table, area, &mut app.nodes_table_state);
    }
}
