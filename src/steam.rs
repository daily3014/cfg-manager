use std::path::PathBuf;
use windows_registry::{CURRENT_USER, Value};

pub fn get_steam_dir() -> Option<PathBuf> {
	let steam_path = CURRENT_USER
		.open("SOFTWARE\\Valve\\Steam")
		.ok()?
		.get_string("SteamPath")
		.ok()?;

	Some(PathBuf::from(steam_path))
}

pub fn get_game_name(id: u32) -> Option<String> {
	CURRENT_USER
		.open(format!("SOFTWARE\\Valve\\Steam\\Apps\\{}", id))
		.and_then(|key| key.get_string("Name"))
		.ok()
}

pub fn get_game_dir(id: u32) -> Option<PathBuf> {
	let steam_dir = get_steam_dir()?;
	let common_path = steam_dir.join("steamapps").join("common");
	let game_path = common_path.join(get_game_name(id)?);

	if game_path.exists() {
		Some(game_path)
	} else {
		None
	}
}

fn get_game_var<T, U>(id: u32, var: &str, closure: U) -> Option<T>
where
	U: FnOnce(Value) -> Option<T>,
{
	let val = CURRENT_USER
		.open(format!("SOFTWARE\\Valve\\Steam\\Apps\\{}", id))
		.and_then(|key| key.get_value(var))
		.ok()?;

	closure(val)
}

pub fn is_game_running(id: u32) -> bool {
	get_game_var(id, "Running", |val| u32::try_from(val).ok()).unwrap_or(0) != 0
}

pub fn is_game_updating(id: u32) -> bool {
	get_game_var(id, "Updating", |val| u32::try_from(val).ok()).unwrap_or(0) != 0
}
