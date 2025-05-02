use crate::{
    cli::utils::{self, SelectMenuResult},
    commands,
};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};

pub async fn menu() -> anyhow::Result<()> {
    loop {
        let options = vec![
            "Back",
            "List Users",
            "Create User",
            "Update User",
            "Delete User",
            "Delete Many Users",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("User Management")
            .default(0)
            .items(&options)
            .interact()?;

        match selection {
            0 => break,
            1 => list_users().await?,
            2 => create_user().await?,
            3 => update_user().await?,
            4 => delete_user().await?,
            5 => delete_many_users().await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn list_users() -> anyhow::Result<()> {
    let mut offset: i64 = 0;
    let limit: i64 = 10;

    loop {
        commands::list_users(limit, offset).await?;

        let actions = vec!["Back", "Next Page", "Previous Page"];
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pagination")
            .items(&actions)
            .default(0)
            .interact()?;

        match choice {
            0 => break,
            1 => offset += limit,
            2 => offset = (offset - limit).max(0),
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn create_user() -> anyhow::Result<()> {
    let username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Username")
        .interact_text()?;
    let email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Email")
        .interact_text()?;
    let password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Password")
        .interact_text()?;

    commands::create_user(username, email, password).await?;
    Ok(())
}

async fn update_user() -> anyhow::Result<()> {
    let user_id_menu_result: SelectMenuResult = utils::select_user()
        .await
        .context("Error selecting user with utils select_user TUI function")?;

    let user_id = match user_id_menu_result {
        SelectMenuResult::Back => return Ok(()),
        SelectMenuResult::Chosen(id) => id,
    };
    let username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("New Username")
        .interact_text()?;
    let email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("New Email")
        .interact_text()?;
    let password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("New Password")
        .interact_text()?;

    commands::update_user(user_id, username, email, password).await?;
    Ok(())
}

async fn delete_user() -> anyhow::Result<()> {
    let user_id_menu_result: SelectMenuResult = utils::select_user()
        .await
        .context("Error selecting user with utils select_user TUI function")?;

    let user_id = match user_id_menu_result {
        SelectMenuResult::Back => return Ok(()),
        SelectMenuResult::Chosen(id) => id,
    };

    commands::delete_user(user_id).await?;
    Ok(())
}

async fn delete_many_users() -> Result<()> {
    // helper to read an i32 or bail on "back"
    async fn prompt_id(prompt: &str) -> Result<Option<i32>> {
        let raw: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{prompt} (or type ‘back’ to cancel)"))
            .interact_text()?;
        if raw.eq_ignore_ascii_case("back") {
            return Ok(None);
        }
        let id = raw
            .parse::<i32>()
            .context(format!("failed to parse `{raw}` as an integer"))?;
        Ok(Some(id))
    }

    // Start ID
    let Some(start_id) = prompt_id("Start ID").await? else {
        // user hit “back”
        return Ok(());
    };

    // End ID
    let Some(end_id) = prompt_id("End ID").await? else {
        // user hit “back”
        return Ok(());
    };

    commands::delete_many_users(start_id, end_id).await?;
    Ok(())
}
