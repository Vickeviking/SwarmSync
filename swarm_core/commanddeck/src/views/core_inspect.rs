use anyhow::Result;
use common::commands::fetch_logs_by_module;
use common::enums::system::SystemModuleEnum;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use std::io;
use std::time::{Duration, Instant};

pub async fn inspect() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let modules = SystemModuleEnum::variants()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut sel = 0;
    let mut log_offset = 0;
    let limit = 5;

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        // fetch logs for module before drawing (avoid block_on)
        let module = SystemModuleEnum::from(sel);
        let logs = fetch_logs_by_module(module.clone(), limit, log_offset)
            .await
            .unwrap_or_default();

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(size);

            // Left pane: modules
            let items: Vec<ListItem> = modules
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let content = if i == sel {
                        Line::from(Span::styled(
                            format!("> {}", m),
                            Style::default().add_modifier(Modifier::BOLD),
                        ))
                    } else {
                        Line::from(Span::raw(m.clone()))
                    };
                    ListItem::new(content)
                })
                .collect();
            let list =
                List::new(items).block(Block::default().title("Modules").borders(Borders::ALL));
            f.render_widget(list, chunks[0]);

            // Right pane: info + logs
            let info = match module {
                SystemModuleEnum::Dispatcher => "Dispatcher: sends tasks to workers.",
                SystemModuleEnum::Harvester => "Harvester: collects results from workers.",
                SystemModuleEnum::Hibernator => "Hibernator: sleeps until cron due.",
                SystemModuleEnum::Receiver => "Receiver: accepts new jobs.",
                SystemModuleEnum::Scheduler => "Scheduler: orders tasks in queue.",
                SystemModuleEnum::TaskArchive => "TaskArchive: stores completed results.",
            };
            let info_para =
                Paragraph::new(info).block(Block::default().title("Info").borders(Borders::ALL));

            let log_items: Vec<ListItem> = logs
                .iter()
                .map(|l| {
                    ListItem::new(Line::from(Span::raw(format!(
                        "{}: {}",
                        l.created_at.format("%Y-%m-%d %H:%M:%S"),
                        l.action
                    ))))
                })
                .collect();
            let logs_list =
                List::new(log_items).block(Block::default().title("Logs").borders(Borders::ALL));

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(chunks[1]);
            f.render_widget(info_para, right_chunks[0]);
            f.render_widget(logs_list, right_chunks[1]);
        })?;

        // input handling remains unchanged
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => sel = (sel + modules.len() - 1) % modules.len(),
                    KeyCode::Down => sel = (sel + 1) % modules.len(),
                    KeyCode::Left => log_offset = log_offset.saturating_sub(limit),
                    KeyCode::Right => log_offset = log_offset.saturating_add(limit),
                    _ => {}
                }
            }
        }
        last_tick = Instant::now();
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
