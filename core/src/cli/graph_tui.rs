use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction as LayoutDirection, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::database::models::{job::Job, job::JobAssignment, worker::Worker};
use crate::shared::enums::job::JobStateEnum;

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
        let mut elements: Vec<(String, Text)> = vec![];
        let mut selectable_indices = vec![];
        let mut graph_lines = vec![];
        let mut connected_worker_ids = vec![];

        let mut index = 0;

        macro_rules! push_line {
            ($line:expr, $text:expr) => {{
                graph_lines.push(Spans::from(Span::raw(format!("   └─ {}", $line))));
                selectable_indices.push(index);
                elements.push(($line, $text));
                index += 1;
            }};
        }

        graph_lines.push(Spans::from(Span::styled(
            format!("👤 User: {}", user_name),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
        index += 1;

        for job in jobs.iter().filter(|j| j.state == JobStateEnum::Submitted) {
            push_line!(
                format!("📦 {} (submitted)", job.job_name),
                Text::from(format!("{}", job))
            );
        }

        graph_lines.push(Spans::from(Span::styled(
            "[Core Queue]",
            Style::default().fg(Color::Yellow),
        )));
        index += 1;

        for job in jobs.iter().filter(|j| j.state == JobStateEnum::Queued) {
            push_line!(
                format!("📦 {} (queued)", job.job_name),
                Text::from(format!("{}", job))
            );
        }

        graph_lines.push(Spans::from(Span::styled(
            "[Workers]",
            Style::default().fg(Color::Cyan),
        )));
        index += 1;

        for job in jobs.iter().filter(|j| j.state == JobStateEnum::Running) {
            if let Some(assign) = assignments.iter().find(|a| a.job_id == job.id) {
                if let Some(worker) = workers.iter().find(|w| w.id == assign.worker_id) {
                    let label = format!("📦 {} → 🛠️ {}", job.job_name, worker.label);
                    push_line!(
                        label,
                        Text::from(format!("{}\n────────────\n{}", worker, job))
                    );
                    connected_worker_ids.push(worker.id);
                    continue;
                }
            }
            push_line!(
                format!("📦 {} (running)", job.job_name),
                Text::from(format!("{}", job))
            );
        }

        for worker in workers {
            if !connected_worker_ids.contains(&worker.id) {
                let label = format!("🛠️ {} (idle)", worker.label);
                push_line!(label.clone(), Text::from(format!("{}", worker)));
            }
        }

        graph_lines.push(Spans::from(Span::styled(
            "[Harvester]",
            Style::default().fg(Color::Blue),
        )));
        index += 1;

        for job in jobs
            .iter()
            .filter(|j| matches!(j.state, JobStateEnum::Completed | JobStateEnum::Failed))
        {
            let icon = if job.state == JobStateEnum::Completed {
                "✅"
            } else {
                "❌"
            };
            push_line!(
                format!("{} {} (done)", icon, job.job_name),
                Text::from(format!("{}", job))
            );
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
                        if Some(i) == selectable_indices.get(selected_index).copied() {
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
