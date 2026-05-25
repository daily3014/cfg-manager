# CFG Manager
## Motivation
I have two configs for low and high graphics. Whenever I want to switch between the two, I go through the process of:
- locating the game directory
- finding the files that the config added in cfg and custom
- moving them to somewhere so i don't lose them
- putting the new config files

And this process starts getting tedious the more often you do it, or if you also like to switch HUDs

You could remedy this by keeping a copy of your custom and cfg folders, where one has config A and other has config B, but then you run into the problem of unnecessarily duplicating files like VPKs and so on

So I wrote this to automatically take care of that for you, by allowing you to store them in one place and then in a single click switch between them without hassle, while also allowing you to define shared files between configs.

## File Structure
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
		my_favourite_hitsound/ - it can distinguish mods from "groups of mods" like above
			sound/
				ui/
					hitsound.wav
				sound.cache

huds/
	ToonHUD
	LightHUD
```
configs, customs and huds are required folders, without them the program won't run! the exe **must be in the same folder as everything else**

The shared folder can hold both groups of mods and regular mods, groups are determined by not containing folders with any of the following names: materials, maps, resource, scripts, sound, models

## How it works
On launch, the app:
- Clears `tf/custom`.
- Adds `customs/shared` into `tf/custom`
- Adds the selected HUD (if any) in `tf/custom`.
- Applies the chosen config preset: copies files from `<config>/<preset>/cfg` into `tf/cfg` when `override_cfg = true`, and `<config>/<preset>/custom` into `tf/custom`.
- Adds the selected customs for the chosen preset from `customs/<preset>`.

The symlinks help to prevent file duplication, and due to the nature of symlinks, changes to those files will persist to the actual file

## Settings
```toml
[settings]
hud = "ToonHUD" # Which HUD to use? Leave empty for none
config = "cfg-tf" # Which config to use? Leave empty for none
preset = "low" # Which preset to use? Used for choosing config and custom presets
override_cfg = false # Whether to override files in tf/cfg if the config preset has files with the same name
```

> [!WARNING]
> Note that `override_cfg` will delete already existing files/folders (autoexec, class specific cfgs, folders like overrides/ and tweaks/ if the config changes them) in favour of symlinks for the benefits listed above

## todo
finish readme