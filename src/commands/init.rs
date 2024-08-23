use crate::{cli_config, commands::vaults::interactive_switch, util::CommandResult};
use clap::Args;
use dialoguer::Confirm;

#[derive(Args, Debug, Clone)]
#[command(arg_required_else_help = true)]
pub struct InitCommand {
    /// Accept all suggestions without prompting
    #[arg(long, short = 'y')]
    yes: Option<bool>,
}

pub fn entry(cmd: &InitCommand) -> CommandResult {
    let updated_config = create_or_overwrite_config(cmd)?;

    if let Some(mut config) = updated_config {
        let next_vault =
            interactive_switch(&config, "Which vault would you like to set as the current");
        config.current_vault = next_vault;
        cli_config::write(config)?
    }

    Ok(None)
}

fn create_or_overwrite_config(cmd: &InitCommand) -> anyhow::Result<Option<cli_config::File>> {
    let config_file_exists = cli_config::exists();
    let config_path = cli_config::get_config_path();

    if config_file_exists {
        let term_is_attended = console::user_attended();
        let override_flag_exists = cmd.yes.is_some();

        let mut confirmation = false;

        if override_flag_exists {
            confirmation = cmd.yes.unwrap_or(false);
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
