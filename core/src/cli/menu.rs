use crate::cli::views::{graph, jobs, logs, users};
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn main_menu() -> anyhow::Result<()> {
    loop {
        let options = vec![
            "Exit",
            "Manage Users",
            "Manage Jobs & Workers",
            "Manage Logs",
            "JobInspect",
        ];

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
            4 => graph::visualize().await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}
