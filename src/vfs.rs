use std::path::Path;

/*
	https://github.com/ValveSoftware/source-sdk-2013/blob/50d5de34e2ae116230fd51683104e2ac9f201565/src/public/filesystem_init.cpp#L814

	materials
	maps
	resource
	scripts
	sound
	models
*/

pub fn dir_is_mod(dir: impl AsRef<Path>) -> bool {
	let dir = dir.as_ref();
	let Ok(entries) = dir.read_dir() else {
		return false;
	};

	entries.flatten().any(|entry| {
		let name = entry.file_name();
		let name_str = name.to_string_lossy();

		matches!(
			name_str.as_ref(),
			"materials" | "maps" | "resource" | "scripts" | "sound" | "models"
		)
	})
}

pub fn file_is_vpk(file: impl AsRef<Path>) -> bool {
	let file = file.as_ref();
	file.extension()
		.is_some_and(|ext| ext.eq_ignore_ascii_case("vpk"))
}
