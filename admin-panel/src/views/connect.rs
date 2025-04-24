use dialoguer::{theme::ColorfulTheme, Input, Select};
use reqwest;

pub async fn choose_core_location() -> anyhow::Result<String> {
    let options = vec!["Local", "Remote"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Where is the Core service running?")
        .items(&options)
        .interact()?;

    let base_url = match selection {
        0 => "http://127.0.0.1:8000".to_string(), // explicit loopback
        1 => {
            let ip: String = Input::new()
                .with_prompt("Enter IP address")
                .interact_text()?;
            format!("http://{}:8000", ip)
        }
        _ => unreachable!(),
    };

    let check = reqwest::get(format!("{}/", base_url)).await;

    match check {
        Ok(resp) if resp.status().is_success() => Ok(base_url),
        _ => {
            println!("❌ Could not connect to Core at {}.", base_url);
            println!("➡ Please make sure the server is running and try again.");
            std::process::exit(1);
        }
    }
}
