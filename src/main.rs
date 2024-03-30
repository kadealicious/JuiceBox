use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_save::SavePlugin;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

pub mod simulation;
pub mod util;
pub mod juice_renderer;
pub mod error;
pub mod file_system;

pub mod test;
pub mod ui;

use util::debug_state_controller;

fn main() {
    let mut juicebox: App = App::new();

	juicebox.add_plugins((
		DefaultPlugins
		.set(util::create_window_plugin())
		.set(AssetPlugin{
			watch_for_changes_override: Some(true),
			..Default::default()
		}),
		simulation::Simulation,
		juice_renderer::JuiceRenderer,
        ui::JuiceUI,
		file_system::FileSystem,
		EguiPlugin,
		SavePlugin,

		// Non-release plugins:
		LogDiagnosticsPlugin::default(),
		FrameTimeDiagnosticsPlugin::default(),
	));

	juicebox.add_systems(Startup, util::set_window_icon);
	juicebox.add_systems(Update, util::control_camera);
	juicebox.add_systems(Update, debug_state_controller);

	juicebox.run();
}
