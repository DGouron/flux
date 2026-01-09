use std::process::Command;

use anyhow::{Context, Result};
use flux_core::{Config, Translator};

pub fn execute() -> Result<()> {
    let translator = get_translator();
    let gui_path = find_flux_gui()?;

    Command::new(&gui_path)
        .spawn()
        .with_context(|| translator.get("error.dashboard_spawn_failed"))?;

    println!("{}", translator.get("command.dashboard_launched"));

    Ok(())
}

fn get_translator() -> Translator {
    Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}

fn find_flux_gui() -> Result<std::path::PathBuf> {
    if let Ok(path) = which::which("flux-gui") {
        return Ok(path);
    }

    if let Ok(current_exe) = std::env::current_exe() {
        let sibling_path = current_exe.with_file_name("flux-gui");
        if sibling_path.exists() {
            return Ok(sibling_path);
        }
    }

    let translator = get_translator();
    anyhow::bail!("{}", translator.get("error.dashboard_not_found"))
}
