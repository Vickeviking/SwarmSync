use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use anyhow::Context;
use common::commands;
use common::database::models::{job::Job, job::JobAssignment, worker::Worker};
use common::enums::job::JobStateEnum;
use common::utils::{self, SelectMenuResult};

pub async fn inspect() -> anyhow::Result<()> {
    let user_id_menu_result: SelectMenuResult = utils::select_user()
        .await
        .context("Error selecting user with utils select_user TUI function")?;

    let user_id = match user_id_menu_result {
        SelectMenuResult::Back => return Ok(()),
        SelectMenuResult::Chosen(id) => id,
    };

    let jobs: Vec<Job> = commands::get_jobs_for_user(user_id)
        .await
        .unwrap_or_default();
    let workers: Vec<Worker> = commands::get_workers_for_user(user_id)
        .await
        .unwrap_or_default();
    let assignments: Vec<JobAssignment> = commands::get_assignments_for_user(user_id)
        .await
        .unwrap_or_default();

    launch_graph_tui_with_data(&jobs, &workers, &assignments)?;
    Ok(())
}

struct ModuleView<'a> {
    title: &'static str,
    items: Vec<(String, Text<'a>)>,
}

pub fn launch_graph_tui_with_data(
    jobs: &[Job],
    workers: &[Worker],
    assignments: &[JobAssignment],
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, jobs, workers, assignments);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<'a, B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    jobs: &'a [Job],
    workers: &'a [Worker],
    assignments: &'a [JobAssignment],
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    let mut module_index = 0;
    let mut job_index = 0;

    loop {
        let mut user_jobs = vec![];
        let mut core_jobs = vec![];
        let mut worker_jobs = vec![];
        let mut connected_worker_ids = vec![];

        for job in jobs {
            let text = Text::from(format!("{}", job));
            match job.state {
                JobStateEnum::Submitted => {
                    user_jobs.push((format!("📦 {} [J{}]", job.job_name, job.id), text));
                }
                JobStateEnum::Queued => {
                    core_jobs.push((format!("📦 {} (queued) [J{}]", job.job_name, job.id), text));
                }
                JobStateEnum::Running => {
                    if let Some(assign) = assignments.iter().find(|a| a.job_id == job.id) {
                        if let Some(worker) = workers.iter().find(|w| w.id == assign.worker_id) {
                            connected_worker_ids.push(worker.id);
                            worker_jobs.push((
                                format!("📦 {} → 🛠️ {} [J{}]", job.job_name, worker.label, job.id),
                                Text::from(format!("{}\n────────────\n{}", worker, job)),
                            ));
                            continue;
                        }
                    }
                    worker_jobs
                        .push((format!("📦 {} (running) [J{}]", job.job_name, job.id), text));
                }
                JobStateEnum::Completed | JobStateEnum::Failed => {
                    let status = if job.state == JobStateEnum::Completed {
                        "✅"
                    } else {
                        "❌"
                    };
                    core_jobs.push((
                        format!("{} {} (done) [J{}]", status, job.job_name, job.id),
                        text,
                    ));
                }
            }
        }

        for w in workers {
            if !connected_worker_ids.contains(&w.id) {
                worker_jobs.push((
                    format!("🛠️ {} (idle) [W{}]", w.label, w.id),
                    Text::from(format!("{}", w)),
                ));
            }
        }

        let views = [
            ModuleView {
                title: "👤 USER",
                items: user_jobs,
            },
            ModuleView {
                title: "⚙️ CORE",
                items: core_jobs,
            },
            ModuleView {
                title: "🛠️ WORKERS",
                items: worker_jobs,
            },
        ];

        terminal.draw(|f| {
            let outer_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(f.size());

            let module_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ])
                .split(outer_chunks[0]);

            for (i, view) in views.iter().enumerate() {
                let items: Vec<Line> = view
                    .items
                    .iter()
                    .enumerate()
                    .map(|(j, (label, _))| {
                        let content = if i == module_index && j == job_index {
                            format!("→ {}", label)
                        } else {
                            label.to_string()
                        };
                        Line::from(Span::raw(content))
                    })
                    .collect();

                let block = Block::default()
                    .title(format!("{} [{}]", view.title, i))
                    .borders(Borders::ALL)
                    .border_style(if i == module_index {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default()
                    });

                let para = Paragraph::new(items).block(block);
                f.render_widget(para, module_chunks[i]);
            }

            let detail_text = views[module_index]
                .items
                .get(job_index)
                .map(|(_, text)| text.clone())
                .unwrap_or_else(|| Text::from("No selection"));

            let detail = Paragraph::new(detail_text).block(
                Block::default()
                    .title("Selected Detail")
                    .borders(Borders::ALL),
            );

            f.render_widget(detail, outer_chunks[1]);
        })?;

        if crossterm::event::poll(tick_rate - last_tick.elapsed())? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => {
                        if module_index > 0 {
                            module_index -= 1;
                            job_index = 0;
                        }
                    }
                    KeyCode::Right => {
                        if module_index < 2 {
                            module_index += 1;
                            job_index = 0;
                        }
                    }
                    KeyCode::Down => {
                        if job_index + 1 < views[module_index].items.len() {
                            job_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        job_index = job_index.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
        last_tick = Instant::now();
    }
    Ok(())
}
