use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::database::models::{job::Job, job::JobAssignment, worker::Worker};

pub fn launch_graph_tui_with_data(
    user_name: &str,
    jobs: &[Job],
    workers: &[Worker],
    assignments: &[JobAssignment],
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, user_name, jobs, workers, assignments);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    user_name: &str,
    jobs: &[Job],
    workers: &[Worker],
    assignments: &[JobAssignment],
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    let mut selected_index = 0;

    loop {
        let mut graph_lines = vec![
            Spans::from(Span::styled(
                format!("👤 User: {}", user_name),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Spans::from(Span::raw("   │")),
            Spans::from(Span::raw("   ▼")),
            Spans::from(Span::styled(
                "[Core System]",
                Style::default().fg(Color::Yellow),
            )),
            Spans::from(Span::raw("   │")),
        ];

        let mut connected_worker_ids = vec![];
        let mut elements: Vec<(String, Text)> = vec![];

        for job in jobs.iter() {
            let mut line = format!("📦 {}", job.job_name);
            let job_detail = format!("{}", job);

            if let Some(assignment) = assignments.iter().find(|a| a.job_id == job.id) {
                if let Some(worker) = workers.iter().find(|w| w.id == assignment.worker_id) {
                    line.push_str(&format!(" → 🛠️ {}", worker.label));
                    connected_worker_ids.push(worker.id);

                    let combined_text = Text::from(format!("{}\n────────────\n{}", worker, job));
                    graph_lines.push(Spans::from(Span::raw(format!("   ├─ {}", line))));
                    elements.push((line, combined_text));
                    continue;
                }
            }

            graph_lines.push(Spans::from(Span::raw(format!("   ├─ {}", line))));
            elements.push((line.clone(), Text::from(job_detail)));
        }

        for worker in workers.iter() {
            if !connected_worker_ids.contains(&worker.id) {
                let line = format!("🛠️ {} (idle)", worker.label);
                graph_lines.push(Spans::from(Span::raw(format!("   └─ {}", line))));
                elements.push((line.clone(), Text::from(format!("{}", worker))));
            }
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(LayoutDirection::Horizontal)
                .margin(2)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(f.size());

            let vertical = Layout::default()
                .direction(LayoutDirection::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(5)])
                .split(chunks[0]);

            let header = Paragraph::new(vec![Spans::from(vec![Span::styled(
                "📊 SwarmSync Graph View",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])]);

            let graph = Paragraph::new(
                graph_lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        if i == selected_index + 5 {
                            Spans::from(Span::styled(
                                line.0.first().unwrap().content.clone(),
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                            ))
                        } else {
                            line.clone()
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .block(Block::default().title("Graph View").borders(Borders::ALL));

            let fallback = Text::from("No selection");
            let selected_detail = elements
                .get(selected_index)
                .map(|(_, d)| d)
                .unwrap_or(&fallback);

            let details = Paragraph::new(selected_detail.clone())
                .block(Block::default().title("Details").borders(Borders::ALL));

            f.render_widget(header, vertical[0]);
            f.render_widget(graph, vertical[1]);
            f.render_widget(details, chunks[1]);
        })?;

        if crossterm::event::poll(tick_rate - last_tick.elapsed())? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if selected_index < elements.len().saturating_sub(1) {
                            selected_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        if selected_index > 0 {
                            selected_index -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        last_tick = Instant::now();
    }

    Ok(())
}
