use crate::state;
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn main_menu() -> anyhow::Result<()> {
    loop {
        let options = vec!["Option1", "Option2", "Logout", "Quit"];
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Main Menu")
            .items(&options)
            .interact()?;

        match choice {
            0 => println!("Option1 (not implemented)"),
            1 => println!("Option2 (not implemented)"),
            2 => {
                println!("Logging out...");
                std::process::exit(0);
            }
            3 => {
                println!("Exiting...");
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
}
