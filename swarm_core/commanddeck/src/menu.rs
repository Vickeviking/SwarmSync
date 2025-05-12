use dialoguer::{theme::ColorfulTheme, Select};

use crate::views::{core_inspect, job_inspect, jobs, logs, users};

/// Entry point for the main menu
pub async fn main_menu() -> anyhow::Result<()> {
    loop {
        // available menu options
        let options = vec![
            "Exit",
            "Manage Users",
            "Manage Jobs & Workers",
            "Manage Logs",
            "JobInspect",
            "CoreInspect",
        ];

        // Ask user for a choice
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Main Menu")
            .default(0)
            .items(&options)
            .interact()?;

        match choice {
            0 => break,
            1 => users::menu().await?,
            2 => jobs::user_perspective_menu().await?,
            3 => logs::menu().await?,
            4 => job_inspect::inspect().await?,
            5 => core_inspect::inspect().await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}
