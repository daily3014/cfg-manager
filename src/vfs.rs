use anyhow::{Context, Result, anyhow};
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::path::Path;

pub fn create_symlink(file: &Path, target: impl AsRef<Path>) -> Result<()> {
	if file.is_dir() {
		symlink_dir(file, target)?;
	} else if file.is_file() {
		symlink_file(file, target)?;
	}

	Ok(())
}

pub fn delete_file(path: &Path) -> Result<()> {
	if let Ok(meta) = path.symlink_metadata() {
		let file_type = meta.file_type();

		if file_type.is_symlink() {
			if path.is_dir() {
				std::fs::remove_dir(path)?;
			} else {
				std::fs::remove_file(path)?;
			}
		} else if file_type.is_dir() {
			std::fs::remove_dir_all(path)?;
		} else if file_type.is_file() {
			std::fs::remove_file(path)?;
		}
	}

	Ok(())
}

pub fn walk_dir<T>(dir: &Path, recurse: bool, closure: &mut T) -> Result<()>
where
	T: FnMut(&Path) -> Result<()>,
{
	if !dir.is_dir() {
		return Err(anyhow!("Path is not a directory: {}", dir.display()));
	}

	for entry in std::fs::read_dir(dir)? {
		let Ok(entry) = entry else { continue };
		let file_type = entry.file_type()?;
		let path = entry.path();

		if file_type.is_dir() && recurse {
			walk_dir(&path, recurse, closure)?;
		} else {
			closure(&path)?;
		}
	}

	Ok(())
}

pub fn copy_dir_as_symlink(dir: &Path, target: &Path) -> Result<()> {
	walk_dir(dir, false, &mut |path| {
		if let Some(file_name) = path.file_name() {
			let target_path = target.join(file_name);

			create_symlink(path, &target_path)
				.with_context(|| format!("Failed to create symlink for '{}'", path.display()))?;
		}

		Ok(())
	})
}

/*
	https://github.com/ValveSoftware/source-sdk-2013/blob/50d5de34e2ae116230fd51683104e2ac9f201565/src/public/filesystem_init.cpp#L814

	materials
	maps
	resource
	scripts
	sound
	models
*/

pub fn dir_is_mod(dir: &Path) -> bool {
	let Ok(entries) = dir.read_dir() else {
		return false;
	};

	entries.flatten().any(|entry| {
		let Ok(file_type) = entry.file_type() else {
			return false;
		};

		if file_type.is_dir() {
			let name = entry.file_name();
			let name_str = name.to_string_lossy();

			matches!(
				name_str.as_ref(),
				"materials" | "maps" | "resource" | "scripts" | "sound" | "models"
			)
		} else {
			false
		}
	})
}

pub fn file_is_vpk(file: &Path) -> bool {
	file.extension()
		.is_some_and(|ext| ext.eq_ignore_ascii_case("vpk"))
}
