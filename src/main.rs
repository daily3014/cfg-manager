use anyhow::{Context, Result, anyhow};
use log::{error, info};
use serde::Deserialize;
use std::{
	iter::once,
	os::windows::fs::{symlink_dir, symlink_file},
	path::{Path, PathBuf},
};
use windows::{
	Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW},
	core::PCWSTR,
};
mod steam;
mod vfs;

#[derive(Deserialize, Debug)]
struct Config {
	settings: Settings,
}

#[derive(Deserialize, Debug)]
struct Settings {
	hud: String,
	config: String,
	preset: String,
	#[serde(rename = "override_cfg")]
	can_override: bool,
}

fn get_directory_from_file(file: impl AsRef<Path>) -> Result<PathBuf> {
	let parent = file
		.as_ref()
		.parent()
		.ok_or(anyhow!("File has no parent"))?;

	parent
		.canonicalize()
		.with_context(|| format!("Failed to canonicalize path: {}", parent.display()))
}

fn parse_config(path: impl AsRef<Path>) -> Result<Config> {
	let content = std::fs::read_to_string(path).context("Failed to read config file")?;
	toml::from_str(&content).context("Invalid configuration format")
}

fn clear_custom(dir: impl AsRef<Path>) {
	if let Ok(entries) = std::fs::read_dir(dir) {
		for entry in entries.flatten() {
			let path = entry.path();

			if let Some(name) = path.file_name()
				&& (name == "workshop" || name == "readme.txt")
				&& path.is_dir()
			{
				continue;
			}

			if path.is_dir() {
				let _ = std::fs::remove_dir_all(path);
			} else if path.is_file() {
				let _ = std::fs::remove_file(path);
			}
		}
	}
}

fn create_symlink(file: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
	let file = file.as_ref();
	let target = target.as_ref();

	if file.is_dir() {
		symlink_dir(file, target)?;
	} else if file.is_file() {
		symlink_file(file, target)?;
	}

	Ok(())
}

fn copy_dir_as_symlink(dir: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
	let dir = dir.as_ref();
	let target = target.as_ref();

	if !dir.is_dir() {
		return Err(anyhow!("Source is not a directory: {}", dir.display()));
	}

	for entry in std::fs::read_dir(dir)? {
		let Ok(entry) = entry else { continue };
		let path = entry.path();
		let target = target.join(entry.file_name());

		create_symlink(&path, &target)
			.with_context(|| format!("Failed to create symlink for '{}'", path.display()))?;
	}

	Ok(())
}

fn copy_cfg_to_tf2(
	preset: impl AsRef<Path>,
	custom_dir: impl AsRef<Path>,
	can_override: bool,
) -> Result<()> {
	let preset = preset.as_ref();
	let custom_dir = custom_dir.as_ref();

	if !preset.is_dir() {
		return Err(anyhow!("Source is not a file: {}", preset.display()));
	}

	let cfg_addons = preset.join("cfg");
	if can_override && cfg_addons.is_dir() {
		for entry in std::fs::read_dir(cfg_addons)? {
			let Ok(entry) = entry else { continue };
			let path = entry.path();
			let target = custom_dir.join(entry.file_name());

			if path.is_file() {
				std::fs::copy(&path, &target)?;
			}
		}
	}

	let custom_addons = preset.join("custom");
	if custom_addons.is_dir() {
		for entry in std::fs::read_dir(custom_addons)? {
			let Ok(entry) = entry else { continue };
			let path = entry.path();
			let target = custom_dir.join(entry.file_name());

			create_symlink(&path, &target)?;
		}
	}

	Ok(())
}

fn run() -> Result<()> {
	let current_dir = get_directory_from_file(std::env::current_exe()?)?;

	let configs_dir = current_dir.join("configs");
	let customs_dir = current_dir.join("customs");
	let huds_dir = current_dir.join("huds");

	for (path, name) in [
		(&configs_dir, "configs"),
		(&customs_dir, "customs"),
		(&huds_dir, "huds"),
	] {
		if !path.is_dir() {
			return Err(anyhow!("{name} directory is missing; cannot continue"));
		}
	}

	let cfg = parse_config(current_dir.join("settings.toml")).context(
		"Could not parse the config file. Ensure that settings.toml exists and wasn't corrupted",
	)?;

	let Some(game_dir) = steam::get_game_dir(440) else {
		return Err(anyhow!(
			"Could not find the game directory, ensure that the game is installed correctly"
		));
	};

	if steam::is_game_running(440) {
		return Err(anyhow!(
			"Game is currently running. Please close it before running this application."
		));
	}

	if steam::is_game_updating(440) {
		return Err(anyhow!(
			"Game is currently updating. Please wait for the update to finish before running this application."
		));
	}

	let tf_custom_dir = game_dir.join("tf").join("custom");
	let tf_cfg_dir = game_dir.join("tf").join("cfg");

	info!("Clearing custom");
	clear_custom(&tf_custom_dir);

	let shared_custom = customs_dir.join("shared");
	if shared_custom.is_dir() {
		info!("Copying shared custom files");
		for entry in std::fs::read_dir(shared_custom)?.flatten() {
			let path = entry.path();

			// For groups of VPKs we need to copy the files since
			// the virtual file system only goes one level deep
			if vfs::dir_is_mod(&path) || vfs::file_is_vpk(&path) {
				create_symlink(&path, &tf_custom_dir)?;
			} else {
				copy_dir_as_symlink(&path, &tf_custom_dir)?;
			}
		}
	}

	if !cfg.settings.hud.is_empty() {
		info!("Adding HUD");
		let hud = huds_dir.join(&cfg.settings.hud);

		if !hud.is_dir() {
			return Err(anyhow!(
				"Specified HUD '{}' does not exist or is not a directory.",
				cfg.settings.hud
			));
		}

		create_symlink(&hud, tf_custom_dir.join(&cfg.settings.hud))
			.context("Failed to create symlink for HUD")?;
	}

	info!("Adding config and custom presets");
	if !cfg.settings.config.is_empty() {
		let config_presets = configs_dir.join(&cfg.settings.config);
		if !config_presets.is_dir() {
			return Err(anyhow!(
				"Specified config '{}' does not exist or is not a directory.",
				cfg.settings.config
			));
		}

		let config_preset = config_presets.join(&cfg.settings.preset);
		if config_preset.is_dir() {
			copy_cfg_to_tf2(config_preset, &tf_cfg_dir, cfg.settings.can_override)?;
		}
	}

	let custom_preset = customs_dir.join(&cfg.settings.preset);
	if custom_preset.is_dir() {
		copy_dir_as_symlink(custom_preset, &tf_custom_dir)?;
	}

	info!("Done!");
	Ok(())
}

fn show_message_box(message: &str, title: &str) {
	let wide_msg: Vec<u16> = message.encode_utf16().chain(once(0)).collect();
	let wide_title: Vec<u16> = title.encode_utf16().chain(once(0)).collect();

	unsafe {
		MessageBoxW(
			None,
			PCWSTR(wide_msg.as_ptr()),
			PCWSTR(wide_title.as_ptr()),
			MB_OK,
		);
	}
}

fn main() {
	env_logger::builder()
		.format_timestamp(None)
		.filter_level(log::LevelFilter::Trace)
		.init();

	match run() {
		Ok(_) => {
			show_message_box("Applied settings", "Success");
		}
		Err(err) => {
			error!("Error: {err}");
			show_message_box(&err.to_string(), "Error");
		}
	}
}
