use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
mod cli;
mod simulation;

fn main() {
    let args = cli::parse();

    App::new()
        .insert_resource(args)
        .add_plugin(simulation::SimulationPlugin)
        .add_plugins(DefaultPlugins)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_plugin(DebugLinesPlugin::default())
        .run();
}
