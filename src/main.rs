use anyhow::{Context, Result, anyhow};
use log::{error, info};
use serde::Deserialize;
use std::{
	iter::once,
	path::{Path, PathBuf},
};
use windows::{
	Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW},
	core::PCWSTR,
};

use crate::vfs::{create_symlink, walk_dir};
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

fn clear_files(tf_custom_dir: &Path, tf_cfg_dir: &Path) -> Result<()> {
	walk_dir(tf_custom_dir, false, &mut |path| {
		if let Some(name) = path.file_name()
			&& (name == "workshop" || name == "readme.txt")
		{
			return Ok(());
		}

		vfs::delete_file(path)
	})?;

	walk_dir(tf_cfg_dir, false, &mut |path| {
		let metadata = path.symlink_metadata()?;

		if metadata.file_type().is_symlink() {
			return vfs::delete_file(path);
		}

		Ok(())
	})
}

fn copy_cfg_to_tf2(preset: &Path, tf_cfg_dir: &Path, can_override: bool) -> Result<()> {
	let cfg_addons = preset.join("cfg");
	let tf_custom_dir = tf_cfg_dir
		.parent()
		.map(|path| path.join("custom"))
		.ok_or(anyhow!("Failed to get parent of tf custom directory"))?;

	if can_override && cfg_addons.is_dir() {
		vfs::walk_dir(&cfg_addons, false, &mut |src| {
			if let Ok(relative) = src.strip_prefix(&cfg_addons) {
				let target = tf_cfg_dir.join(relative);

				// Symlinking will allow us to later delete them easily
				vfs::delete_file(&target)?; // if it already exists, we need to delete it,
				create_symlink(src, &target)?;
			}
			Ok(())
		})?;
	}

	let custom_addons = preset.join("custom");
	if custom_addons.is_dir() {
		vfs::walk_dir(&custom_addons, false, &mut |src| {
			if let Some(file_name) = src.file_name() {
				let target = tf_custom_dir.join(file_name);

				vfs::create_symlink(src, &target)?;
			}
			Ok(())
		})?;
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

	let game_dir = steam::get_game_dir(440).context(
		"Could not find the game directory, ensure that the game is installed correctly",
	)?;

	if steam::is_game_running(440) {
		return Err(anyhow!(
			"Game is currently running. Please close it before running cfg-manager."
		));
	}

	if steam::is_game_updating(440) {
		return Err(anyhow!(
			"Game is currently updating. Please wait for the update to finish before running cfg-manager."
		));
	}

	let tf_custom_dir = game_dir.join("tf").join("custom");
	let tf_cfg_dir = game_dir.join("tf").join("cfg");

	info!("Clearing custom and cfg");
	clear_files(&tf_custom_dir, &tf_cfg_dir)?;

	let shared_custom = customs_dir.join("shared");
	if shared_custom.is_dir() {
		info!("Copying shared custom files");
		vfs::walk_dir(&shared_custom, false, &mut |path| {
			if vfs::dir_is_mod(path) || vfs::file_is_vpk(path) {
				vfs::create_symlink(path, &tf_custom_dir)?;
			} else {
				vfs::copy_dir_as_symlink(path, &tf_custom_dir)?;
			}

			Ok(())
		})?;
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

		vfs::create_symlink(&hud, &tf_custom_dir.join(&cfg.settings.hud))
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
			copy_cfg_to_tf2(&config_preset, &tf_cfg_dir, cfg.settings.can_override)?;
		}
	}

	let custom_preset = customs_dir.join(&cfg.settings.preset);
	if custom_preset.is_dir() {
		vfs::copy_dir_as_symlink(&custom_preset, &tf_custom_dir)?;
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
