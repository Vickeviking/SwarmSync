use crate::cli::views::{graph, jobs, users};
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn main_menu() -> anyhow::Result<()> {
    loop {
        let options = vec![
            "Exit",
            "Manage Users",
            "Manage Jobs & Workers",
            "System Graph View",
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
            3 => graph::visualize().await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}
