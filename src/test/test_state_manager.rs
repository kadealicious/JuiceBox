use bevy::prelude::*;
use crate::juice_renderer::draw_selection_circle;
use crate::simulation::sim_state_manager::{
	select_particles,
	delete_particle,
};
use crate::simulation::SimGridCellType;
use crate::simulation::{
	SimConstraints,
	SimParticle,
	SimGrid,
	sim_state_manager::{
		add_particles_in_radius,
		add_particle,
	},
};
use crate::util::get_cursor_position;

/// Create a simulation layout for testing.
pub fn construct_test_simulation_layout(
	constraints:	&mut SimConstraints,
	grid:			&mut SimGrid,
	mut commands:	Commands) {

	// Create a lil cup
	grid.set_grid_cell_type(40, 32, SimGridCellType::Solid);
	grid.set_grid_cell_type(41, 32, SimGridCellType::Solid);
	grid.set_grid_cell_type(42, 32, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 32, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 33, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 34, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 35, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 36, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 37, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 38, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 39, SimGridCellType::Solid);
	grid.set_grid_cell_type(42, 39, SimGridCellType::Solid);
	grid.set_grid_cell_type(41, 39, SimGridCellType::Solid);
	grid.set_grid_cell_type(40, 39, SimGridCellType::Solid);

	// Create a BIG wall
	grid.set_grid_cell_type(40, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(41, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(42, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(43, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(44, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(45, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(46, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(47, 14, SimGridCellType::Solid);
	grid.set_grid_cell_type(48, 14, SimGridCellType::Solid);

    for i in 0..50 {
        grid.set_grid_cell_type(49, i, SimGridCellType::Solid);
        grid.set_grid_cell_type(0, i, SimGridCellType::Solid);
        grid.set_grid_cell_type(i, 0, SimGridCellType::Solid);
        grid.set_grid_cell_type(i, 49, SimGridCellType::Solid);
    }


	// Spawn a small test group of particles at the center of the screen.
	let grid_center: Vec2 = Vec2 {
		x: (grid.dimensions.1 * grid.cell_size) as f32 * 0.5,
		y: (grid.dimensions.0 * grid.cell_size) as f32 * 0.5,
	};

	// let _ = add_particle(
	// 	&mut commands,
	// 	constraints,
	// 	grid,
	// 	Vec2 { x: grid_center[0] * 1.5, y: grid_center[1] * 0.75 },
	// 	Vec2 { x: 0.0, y: 0.0 }
	// );

	let _moar_test_particles = add_particles_in_radius(
		&mut commands,
        constraints,
		grid,
		1.5,
		100.0,
		Vec2 { x: grid_center[0] * 1.5, y: grid_center[1] * 0.75 },
		Vec2::ZERO
	);

	println!("Creating a test simulation with {} particles...", constraints.particle_count);
}

/// Test particle selection.
pub fn test_select_particles(
	commands:			&mut Commands,
	constraints:		&mut SimConstraints,
	mut grid:			&mut SimGrid,
	mut particles:		&Query<(Entity, &mut SimParticle)>,
	windows:			&Query<&Window>,
	cameras:			&Query<(&Camera, &GlobalTransform)>,
	mut gizmos:			&mut Gizmos) {

	let radius: f32				= 55.0;
	let cursor_position: Vec2	= get_cursor_position(windows, cameras);
	draw_selection_circle(gizmos, cursor_position, radius, Color::YELLOW);

	// Test particle selection.
	let selected_particles: Vec<Entity> = select_particles(
		particles,
		grid,
		cursor_position,
		radius
	);

	// Delete them to prove it worked!
	for particle in selected_particles.iter() {
		let _ = delete_particle(commands, constraints, particles, grid, *particle);
	}
}

/// Test grid cell selection.
pub fn test_select_grid_cells(
	commands:			&mut Commands,
	constraints:		&mut SimConstraints,
	mut grid:			&mut SimGrid,
	mut particles:		&Query<(Entity, &mut SimParticle)>,
	windows:			&Query<&Window>,
	cameras:			&Query<(&Camera, &GlobalTransform)>,
	mut gizmos:			&mut Gizmos) {

	let radius: f32				= 55.0;
	let cursor_position: Vec2	= get_cursor_position(windows, cameras);

	// Test cell selection.
	let selected_cells: Vec<Vec2> = grid.select_grid_cells(cursor_position, radius);
	for i in 0..selected_cells.len() {

		let half_cell_size: f32 = grid.cell_size as f32 * 0.5;
		let mut cell_position: Vec2 = grid.get_cell_position_from_coordinates(selected_cells[i]);

		cell_position.x += half_cell_size;
		cell_position.y += half_cell_size;

		gizmos.rect_2d(
			cell_position,
			0.0,
			Vec2 {
				x: grid.cell_size as f32,
				y: grid.cell_size as f32,
			},
			Color::ANTIQUE_WHITE
		);
	}
}
