# CFG Manager

## Example File Structure
```
settings.toml - contains which config and custom preset to load
configs/
	cfg-tf/
		low - low preset config
		high - high preset config
	mastercomfig/
		low - low preset config
		high - high preset config
customs/
	low/my_mod.vpk - mods for low preset
	high/my_other_mod.vpk - mods for high preset
	shared/ - mods shared between all presets
		shared_mod.vpk
		group_of_vpks/
			vpk1.vpk
			vpk2.vpk
		hitsound/
			sound/
				ui/
					hitsound.wav
				sound.cache

huds/
	ToonHUD
	LightHUD
```
The shared folder can hold both groups of mods and regular mods, groups are determined by not containing folders with any of the following names: materials, maps, resource, scripts, sound, models

## How it works
On launch, the app:
- Clears `tf/custom`.
- Adds `customs/shared` into `tf/custom` using symlinks; folders that are not mods are linked one level deep to avoid VFS issues.
- Adds the selected HUD (if any) as a symlink in `tf/custom`.
- Applies the chosen config preset: copies files from `<config>/<preset>/cfg` into `tf/cfg` when `override_cfg = true`, and symlinks `<config>/<preset>/custom`.
- Adds the selected custom preset from `customs/<preset>` as symlinks.

## Settings
```toml
[settings]
hud = "ToonHUD" # Which HUD to use? Leave empty for none
config = "cfg-tf" # Which config to use? Leave empty for none
preset = "low" # Which preset to use? Used for choosing config and custom presets
override_cfg = false # Whether to override files in tf/cfg if the config preset has files with the same name
```

## todo
finish readme