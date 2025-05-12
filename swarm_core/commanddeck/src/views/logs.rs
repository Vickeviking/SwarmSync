use anyhow::Result;
use chrono::{Local, NaiveDateTime};
use dialoguer::{theme::ColorfulTheme, Input, Select};

use common::{
    commands::{
        create_log_entry, delete_log_entry, fetch_log_entry, fetch_logs, fetch_logs_by_action,
        fetch_logs_by_level, fetch_logs_by_module, update_log_entry,
    },
    database::models::log::DBLogEntry,
    enums::{log::LogActionEnum, log::LogLevelEnum, system::SystemModuleEnum},
};

// Logs main menu
pub async fn menu() -> Result<()> {
    loop {
        let items = vec!["Back", "Create Log", "Browse Logs"];
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Logs Menu")
            .items(&items)
            .default(0)
            .interact()?;

        match choice {
            0 => break,
            1 => create_flow().await?,
            2 => browse_flow().await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

// Create Log Flow
async fn create_flow() -> Result<()> {
    let level = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Level")
        .items(LogLevelEnum::variants())
        .interact()?
        .into();
    let module = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Module")
        .items(SystemModuleEnum::variants())
        .interact()?
        .into();
    let action = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Action")
        .items(LogActionEnum::variants())
        .interact()?
        .into();
    let expires_at = Local::now().naive_local();

    let raw_ip: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Client IP (or blank)")
        .allow_empty(true)
        .interact_text()?;
    let client_ip = if raw_ip.trim().is_empty() {
        None
    } else {
        Some(raw_ip)
    };

    let raw_user: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Client Username (or blank)")
        .allow_empty(true)
        .interact_text()?;
    let client_username = if raw_user.trim().is_empty() {
        None
    } else {
        Some(raw_user)
    };

    let raw_msg: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Custom Message (or blank)")
        .allow_empty(true)
        .interact_text()?;
    let custom_msg = if raw_msg.trim().is_empty() {
        None
    } else {
        Some(raw_msg)
    };

    create_log_entry(
        level,
        module,
        action,
        expires_at,
        client_ip,
        client_username,
        custom_msg,
    )
    .await?;
    Ok(())
}

// Browse Log Flow
async fn id_flow() -> Result<()> {
    let id: i32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Log ID")
        .interact_text()?;
    manage_single_log(id).await
}

// Browse Log Flow, Browse menu
async fn browse_flow() -> Result<()> {
    loop {
        let opts = vec![
            "Back",
            "Get by ID",
            "All Logs",
            "By Action",
            "By Level",
            "By Module",
        ];
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Browse Logs")
            .items(&opts)
            .default(0)
            .interact()?;
        match choice {
            0 => break,
            1 => id_flow().await?,
            2 => browse_all().await?,
            3 => browse_action().await?,
            4 => browse_level().await?,
            5 => browse_module().await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

/// Browse all logs
async fn browse_all() -> Result<()> {
    let mut offset = 0;
    let limit = 10;
    loop {
        let logs = fetch_logs(limit, offset).await?;
        if logs.is_empty() {
            println!("📭 No logs.");
            break;
        }
        if select_and_manage(&logs).await? {
            break;
        }
        offset = paginate_offset(offset, limit)?;
    }
    Ok(())
}

/// Browse logs by action
async fn browse_action() -> Result<()> {
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Action")
        .items(LogActionEnum::variants())
        .interact()?;
    let action: LogActionEnum = idx.into();

    let mut offset = 0;
    let limit = 10;
    loop {
        let logs = fetch_logs_by_action(action.clone(), limit, offset).await?;
        if logs.is_empty() {
            println!("📭 No logs for action `{}`.", action);
            break;
        }
        if select_and_manage(&logs).await? {
            break;
        }
        offset = paginate_offset(offset, limit)?;
    }
    Ok(())
}

/// Browse logs by level
async fn browse_level() -> Result<()> {
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Level")
        .items(LogLevelEnum::variants())
        .interact()?;
    let level: LogLevelEnum = idx.into();

    let mut offset = 0;
    let limit = 10;
    loop {
        let logs = fetch_logs_by_level(level.clone(), limit, offset).await?;
        if logs.is_empty() {
            println!("📭 No logs for level `{}`.", level);
            break;
        }
        if select_and_manage(&logs).await? {
            break;
        }
        offset = paginate_offset(offset, limit)?;
    }
    Ok(())
}

/// Browse logs by module
async fn browse_module() -> Result<()> {
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Module")
        .items(SystemModuleEnum::variants())
        .interact()?;
    let module: SystemModuleEnum = idx.into();

    let mut offset = 0;
    let limit = 10;
    loop {
        let logs = fetch_logs_by_module(module.clone(), limit, offset).await?;
        if logs.is_empty() {
            println!("📭 No logs for module {:?}.", module);
            break;
        }
        if select_and_manage(&logs).await? {
            break;
        }
        offset = paginate_offset(offset, limit)?;
    }
    Ok(())
}

/// Select and manage a single log
/// # Arguments
/// * logs - List of logs
async fn select_and_manage(logs: &[DBLogEntry]) -> Result<bool, anyhow::Error> {
    for entry in logs {
        println!(
            "({}) [{}] {} – {}",
            entry.id, entry.level, entry.module, entry.action
        );
    }
    let mut items: Vec<String> = logs.iter().map(|l| l.id.to_string()).collect();
    items.extend_from_slice(&["Next Page".into(), "Prev Page".into(), "Back".into()]);
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select ID or navigate")
        .items(&items)
        .interact()?;
    let n = logs.len();
    if choice < n {
        manage_single_log(logs[choice].id).await?;
        return Ok(true);
    }
    match choice - n {
        0 => { /* Next Page handled by caller */ }
        1 => { /* Prev Page */ }
        2 => return Ok(true),
        _ => unreachable!(),
    }
    Ok(false)
}

/// Paginate offset
/// # Arguments
/// * `offset` - Current offset
/// * `limit` - Current limit
fn paginate_offset(offset: i64, limit: i64) -> Result<i64, anyhow::Error> {
    let actions = vec!["Next Page", "Prev Page"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Page controls")
        .items(&actions)
        .default(0)
        .interact()?;
    Ok(match choice {
        0 => offset + limit,
        1 => offset.saturating_sub(limit),
        _ => offset,
    })
}

/// Single log edit
/// # Arguments
/// * `id` - ID of the log to edit
async fn manage_single_log(id: i32) -> Result<(), anyhow::Error> {
    let actions = vec!["Back", "Edit", "Delete"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Log {} actions", id))
        .items(&actions)
        .default(0)
        .interact()?;

    if choice == 1 {
        // Edit flow: load existing entry
        let mut entry = fetch_log_entry(id).await?;

        // Level: keep or change
        let mut level_items: Vec<String> = vec![format!("Keep current ({})", entry.level)];
        level_items.extend(LogLevelEnum::variants().iter().map(|s| s.to_string()));
        let lvl_sel = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select new level or keep current")
            .items(&level_items)
            .default(0)
            .interact()?;
        if lvl_sel > 0 {
            entry.level = (lvl_sel - 1).into();
        }

        // Module: keep or change
        let mut module_items: Vec<String> = vec![format!("Keep current ({})", entry.module)];
        module_items.extend(SystemModuleEnum::variants().iter().map(|s| s.to_string()));
        let mdl_sel = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select new module or keep current")
            .items(&module_items)
            .default(0)
            .interact()?;
        if mdl_sel > 0 {
            entry.module = (mdl_sel - 1).into();
        }

        // Action: keep or change
        let mut action_items: Vec<String> = vec![format!("Keep current ({})", entry.action)];
        action_items.extend(LogActionEnum::variants().iter().map(|s| s.to_string()));
        let act_sel = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select new action or keep current")
            .items(&action_items)
            .default(0)
            .interact()?;
        if act_sel > 0 {
            entry.action = (act_sel - 1).into();
        }

        // Expires At: prompt with default, blank = keep
        let exp_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Expires At [{}] (YYYY-MM-DD HH:MM:SS)",
                entry.expires_at
            ))
            .allow_empty(true)
            .interact_text()?;
        if !exp_str.trim().is_empty() {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&exp_str, "%Y-%m-%d %H:%M:%S") {
                entry.expires_at = dt;
            }
        }

        // Client IP: blank = keep
        let ip_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Client IP [{:?}]", entry.client_connected_ip))
            .allow_empty(true)
            .interact_text()?;
        if !ip_str.trim().is_empty() {
            entry.client_connected_ip = Some(ip_str);
        }

        // Client Username
        let user_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Client Username [{:?}]",
                entry.client_connected_username
            ))
            .allow_empty(true)
            .interact_text()?;
        if !user_str.trim().is_empty() {
            entry.client_connected_username = Some(user_str);
        }

        // Custom message
        let msg_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Custom Msg [{:?}]", entry.custom_msg))
            .allow_empty(true)
            .interact_text()?;
        if !msg_str.trim().is_empty() {
            entry.custom_msg = Some(msg_str);
        }

        // Perform update
        update_log_entry(id, entry).await?;
    } else if choice == 2 {
        // Delete
        delete_log_entry(id).await?;
    }

    Ok(())
}
