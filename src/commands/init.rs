use crate::{
    cli_config,
    commands::vaults::interactive_switch,
    util::{should_enable_interactivity, CommandResult},
};
use clap::{ArgAction, Args};
use dialoguer::Confirm;

#[derive(Args, Debug, Clone)]
pub struct InitCommand {
    /// Accept all suggestions without prompting
    #[arg(long, action=ArgAction::SetTrue)]
    overwrite: bool,

    /// Don't prompt to select a vault
    #[arg(long, action=ArgAction::SetTrue)]
    auto_vault: bool,
}

pub fn entry(cmd: &InitCommand) -> CommandResult {
    let updated_config = create_or_overwrite_config(cmd)?;

    if let Some(mut config) = updated_config {
        if !cmd.auto_vault {
            let next_vault =
                interactive_switch(&config, "Which vault would you like to set as the current");
            config.current_vault = next_vault;
        }
        cli_config::write(&config)?;
    }

    Ok(None)
}

fn create_or_overwrite_config(cmd: &InitCommand) -> anyhow::Result<Option<cli_config::Config>> {
    let config_file_exists = cli_config::exists();
    let config_path = cli_config::get_config_path();

    if config_file_exists {
        let term_is_attended = should_enable_interactivity();

        let mut confirmation = false;

        if cmd.overwrite {
            confirmation = true;
        } else if term_is_attended {
            let prompt = format!(
                "A config file already exists at {}, do you want to override it?",
                &config_path.display()
            );
            confirmation = Confirm::new().with_prompt(prompt).interact()?;
        }

        if confirmation {
            let config = cli_config::create_from_settings()?;
            println!("Config file overwritten");
            Ok(Some(config))
        } else {
            println!("Config file left as-is");
            Ok(None)
        }
    } else {
        let config = cli_config::create_from_settings()?;
        println!("Config file created at {}", config_path.display());
        Ok(Some(config))
    }
}
