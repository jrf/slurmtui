use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{cmp_node_col, App, NodeSort, SortDir};
use crate::colors;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    app.nodes_viewport = area.height.saturating_sub(3).max(1);
    let mut partitions: Vec<_> = app.partitions.iter().collect();
    let (ncol, ndir) = app.nodes_sort;
    partitions.sort_by(|a, b| {
        let ord = cmp_node_col(a, b, ncol);
        if ndir == SortDir::Asc {
            ord
        } else {
            ord.reverse()
        }
    });
    let title = format!(" Partitions ({}) ", partitions.len());

    let (sort_col, sort_dir) = app.nodes_sort;
    let hdr = |label: &str, col: NodeSort| -> String {
        if col == sort_col {
            format!("{} {}", label, sort_dir.arrow())
        } else {
            label.to_string()
        }
    };

    let col_specs = [
        (
            "Partition",
            Some(NodeSort::Partition),
            Constraint::Length(14),
            0,
        ),
        ("Avail", Some(NodeSort::Avail), Constraint::Length(6), 0),
        (
            "TimeLimit",
            Some(NodeSort::TimeLimit),
            Constraint::Length(14),
            70,
        ),
        ("Nodes", Some(NodeSort::Nodes), Constraint::Length(6), 0),
        ("State", Some(NodeSort::State), Constraint::Length(6), 0),
        ("CPUs", Some(NodeSort::Cpus), Constraint::Length(6), 85),
        ("Mem(GB)", Some(NodeSort::Memory), Constraint::Length(8), 95),
        ("GRES", None, Constraint::Length(18), 110),
        ("NodeList", None, Constraint::Fill(1), 0),
    ];

    let visible_cols: Vec<_> = col_specs
        .iter()
        .filter(|&&(_, _, _, min_w)| area.width >= min_w)
        .collect();

    let header_cells = visible_cols.iter().map(|&(label, sort_key, _, _)| {
        if let Some(col) = sort_key {
            Cell::from(hdr(label, *col))
        } else {
            Cell::from(label.to_string())
        }
    });

    let header = Row::new(header_cells).style(
        Style::default()
            .fg(colors::blue())
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = partitions
        .iter()
        .map(|p| {
            let state_color = match p.state.as_str() {
                "idle" => colors::green(),
                "mix" => colors::purple(),
                "alloc" | "allocated" => colors::blue1(),
                "down" | "down*" => colors::red(),
                "drain" | "drng" => colors::comment(),
                _ => colors::fg(),
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
                Style::default().fg(colors::fg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::fg())
            };

            let mut cells = Vec::with_capacity(visible_cols.len());
            for &(label, _, _, _) in &visible_cols {
                let cell = match *label {
                    "Partition" => Cell::from(Span::styled(p.partition.as_str(), partition_style)),
                    "Avail" => Cell::from(Span::styled(
                        p.avail.as_str(),
                        Style::default().fg(colors::fg_dark()),
                    )),
                    "TimeLimit" => Cell::from(Span::styled(
                        p.time_limit.as_str(),
                        Style::default().fg(colors::dark5()),
                    )),
                    "Nodes" => Cell::from(Span::styled(
                        p.nodes.to_string(),
                        Style::default().fg(colors::magenta()),
                    )),
                    "State" => Cell::from(Span::styled(
                        p.state.as_str(),
                        Style::default().fg(state_color),
                    )),
                    "CPUs" => Cell::from(Span::styled(
                        p.cpus_per_node.to_string(),
                        Style::default().fg(colors::magenta()),
                    )),
                    "Mem(GB)" => Cell::from(Span::styled(
                        mem_gb.clone(),
                        Style::default().fg(colors::magenta()),
                    )),
                    "GRES" => Cell::from(Span::styled(
                        gres_display.clone(),
                        Style::default().fg(colors::cyan()),
                    )),
                    "NodeList" => Cell::from(Span::styled(
                        p.nodelist.as_str(),
                        Style::default().fg(colors::fg_dark()),
                    )),
                    _ => Cell::from(""),
                };
                cells.push(cell);
            }
            Row::new(cells)
        })
        .collect();

    if rows.is_empty() {
        let msg = Paragraph::new(
            Line::from(Span::styled(
                "No partition data",
                Style::default().fg(colors::comment()),
            ))
            .centered(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::fg_gutter()))
                .title(Span::styled(title, Style::default().fg(colors::blue()))),
        );
        frame.render_widget(msg, area);
    } else {
        let widths: Vec<Constraint> = visible_cols
            .iter()
            .map(|&(_, _, width, _)| *width)
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::fg_gutter()))
                    .title(Span::styled(title, Style::default().fg(colors::blue()))),
            )
            .row_highlight_style(
                Style::default()
                    .bg(colors::bg_highlight())
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(table, area, &mut app.nodes_table_state);
    }
}
