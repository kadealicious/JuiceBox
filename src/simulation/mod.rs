pub mod sim_physics_engine;
pub mod sim_state_manager;
pub mod util;

use bevy::prelude::*;
use bevy::math::Vec2;
use crate::{error::Error, test::test_state_manager::test_select_grid_cells};
use sim_physics_engine::*;
use crate::test::test_state_manager;

use self::{sim_state_manager::{
	delete_particle,
	delete_all_particles
}, util::find_influence};

pub type Result<T> = core::result::Result<T, Error>;

pub struct Simulation;
impl Plugin for Simulation {

	fn build(&self, app: &mut App) {
		app.insert_resource(SimConstraints::default());
		app.insert_resource(SimGrid::default());

		app.add_systems(Startup, setup);
		app.add_systems(Update, update);
	}
}

/// Simulation state manager initialization.
fn setup(
	commands:			Commands,
	mut constraints:	ResMut<SimConstraints>,
	mut grid:			ResMut<SimGrid>) {
	
	test_state_manager::construct_test_simulation_layout(
		constraints.as_mut(),
		grid.as_mut(),
		commands
	);


	// TODO: Get saved simulation data from most recently open file OR default file.
	// TODO: Population constraints, grid, and particles with loaded data.
}

/// Simulation state manager update; handles user interactions with the simulation.
fn update(
	mut constraints:	ResMut<SimConstraints>,
	mut grid:			ResMut<SimGrid>,
	mut particles:		Query<(Entity, &mut SimParticle)>,
	keys:				Res<Input<KeyCode>>,
	
	mut commands:	Commands,
	mut gizmos:		Gizmos,
	windows:		Query<&Window>,
	cameras:		Query<(&Camera, &GlobalTransform)>
	) {

	// TODO: Check for and handle simulation saving/loading.
	// TODO: Check for and handle simulation pause/timestep change.
	
	// let delta_time: f32 = time.delta().as_millis() as f32 * 0.001;
	let fixed_timestep: f32 = constraints.timestep;
	
	// If F is not being held, run the simulation.
	if !keys.pressed(KeyCode::F) {
		step_simulation_once(
			constraints.as_mut(),
			grid.as_mut(),
			&mut particles,
			fixed_timestep
		);

		// If F is being held and G is tapped, step the simulation once.
	} else if keys.just_pressed(KeyCode::G) {
		step_simulation_once(
			constraints.as_mut(),
			grid.as_mut(),
			&mut particles,
			fixed_timestep
		);
	}
	
	// test_select_grid_cells(
	// 	&mut commands,
	// 	constraints.as_mut(),
	// 	grid.as_mut(),
	// 	&particles,
	// 	&windows,
	// 	&cameras,
	// 	&mut gizmos
	// );

	// TODO: Check for and handle changes to gravity.
	// TODO: Check for and handle tool usage.
}

/// Step the fluid simulation one time!
fn step_simulation_once(
	constraints:	&mut SimConstraints,
	grid:			&mut SimGrid,
	particles:		&mut Query<(Entity, &mut SimParticle)>,
	timestep:		f32) {

    update_particles(constraints, particles, grid, timestep);
    push_particles_apart(constraints, grid, particles, timestep);
    handle_particle_collisions(constraints, grid, particles, timestep);
    let old_grid: SimGrid = particles_to_grid(grid, particles);
    make_grid_velocities_incompressible(grid, constraints);
    let change_grid = create_change_grid(&old_grid, &grid);
    grid_to_particles(grid, &change_grid, particles, constraints.grid_particle_ratio);
}

/// Reset simulation components to their default state and delete all particles.
pub fn reset_simulation_to_default(
	commands:			&mut Commands,
	mut constraints:	&mut SimConstraints,
	mut grid:			&mut SimGrid,
	particles:			&Query<(Entity, &mut SimParticle)>) {

	println!("Resetting simulation to default...");
	delete_all_particles(commands, constraints, grid, particles);
	*grid			= SimGrid::default();
	*constraints	= SimConstraints::default();
}

#[derive(Resource)]
pub struct SimConstraints {
	pub grid_particle_ratio:		f32, 	// PIC/FLIP simulation ratio (0.0 = FLIP, 1.0 = PIC).
	pub timestep:					f32,	// Timestep for simulation updates.
	pub incomp_iters_per_frame:		u8, 	// Simulation incompressibility iterations per frame.
	pub collision_iters_per_frame:	u8,		// Collision iterations per frame.
	pub gravity:					Vec2,	// Cartesian gravity vector.
	pub particle_radius:			f32,	// Particle collision radii.
	pub particle_count:				usize,	// Number of particles in the simulation.
	pub particle_rest_density:		f32,	// Rest density of particles in simulation.
}

impl Default for SimConstraints {

	fn default() -> SimConstraints {
		SimConstraints {
			grid_particle_ratio:		0.0,	// 0.0 = inviscid (FLIP), 1.0 = viscous (PIC).
			timestep:					1.0 / 120.0,
			incomp_iters_per_frame:		2,
			collision_iters_per_frame:	2,
			gravity:					Vec2 { x: 0.0, y: -96.0 },
			particle_radius:			1.5,
			particle_count:				0,
			particle_rest_density:		0.0,
		}
	}
}

impl SimConstraints {
	/// Change the gravity direction and strength constraints within the simulation.
	fn change_gravity(sim: &mut SimConstraints, gravity: Vec2) {
		sim.gravity = gravity;
	}

	// Toggle Timestep from defualt and zero value
	fn toggle_simulation_pause(sim: &mut SimConstraints) {
		if sim.incomp_iters_per_frame != 0 {
			sim.incomp_iters_per_frame = 0;
		}
		else{
			sim.incomp_iters_per_frame = 5;
            // TODO: Create a variable to represent last speed set by user
		}
	}

	// Changes number of iterations for incompressibility per frame.
	fn change_incompressibility_timestep(sim: &mut SimConstraints, new_timstep: u8) {
		sim.incomp_iters_per_frame = new_timstep;
	}

	// Changes number of iterations for particle collision per frame.
	fn change_collision_timestep(sim: &mut SimConstraints, new_timstep: u8) {
		sim.collision_iters_per_frame = new_timstep;
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimGridCellType {
	Solid,
    Fluid,
	Air,
}

#[derive(Resource, Clone)]
pub struct SimGrid {
	pub	dimensions:	    (u16, u16),				// # of Hor. and Vert. cells in the simulation.
	pub	cell_size:		u16,
	pub	cell_type:		Vec<Vec<SimGridCellType>>,
	pub cell_center:    Vec<Vec<f32>>,			// Magnitude of pressure at center of cell.
	pub	velocity_u:		Vec<Vec<f32>>,			// Hor. magnitude as row<column<>>; left -> right.
	pub velocity_v:     Vec<Vec<f32>>,			// Vert. magnitude as row<column<>>; up -> down.
	pub spatial_lookup:	Vec<Vec<Entity>>,		// [cell_hash_value[list_of_entities_within_cell]].
	pub density:		Vec<f32>,				// Density for each grid cell.
}

impl Default for SimGrid {

	fn default() -> SimGrid {
		SimGrid {
			dimensions:	    (100, 100),
			cell_size:		5,
			cell_type:		vec![vec![SimGridCellType::Air; 100]; 100],
            cell_center:    vec![vec![0.0; 100]; 100],
			velocity_u:		vec![vec![0.0; 101]; 100],
            velocity_v:     vec![vec![0.0; 100]; 101],
			spatial_lookup:	vec![vec![Entity::PLACEHOLDER; 0]; 10000],
			density:		vec![0.0; 10000],
		}
	}
}

impl SimGrid {

	/// Create a new SimGrid!
	fn change_dimensions(&mut self, dimensions: (u16, u16), cell_size: u16) {

		let row_count: usize	= dimensions.0 as usize;
		let col_count: usize	= dimensions.1 as usize;

		self.dimensions			= dimensions;
		self.cell_size			= cell_size;
		self.cell_type			= vec![vec![SimGridCellType::Air; row_count]; col_count];
		self.cell_center		= vec![vec![0.0; row_count]; col_count];
		self.velocity_u			= vec![vec![0.0; row_count + 1]; col_count];
		self.velocity_v			= vec![vec![0.0; row_count]; col_count + 1];
		self.spatial_lookup		= vec![vec![Entity::PLACEHOLDER; 0]; row_count * col_count];
		self.density			= vec![0.0; row_count * col_count];
	}

	/// Set simulation grid cell type.
    pub fn set_grid_cell_type(
        &mut self,
        cell_x: usize,
		cell_y: usize,
        cell_type: SimGridCellType) -> Result<()> {

		if cell_x >= self.dimensions.0 as usize {
			return Err(Error::OutOfGridBounds("X-coord. is out of bounds!"));
		}
		if cell_y >= self.dimensions.1 as usize {
			return Err(Error::OutOfGridBounds("Y-coord. is out of bounds!"));
		}

        self.cell_type[cell_x][cell_y] = cell_type;

        Ok(())
    }

	/// Set simulation grid dimensions.
    pub fn set_grid_dimensions(
        &mut self,
        width: u16,
        height: u16) -> Result<()> {

        self.dimensions = (width, height);

        Ok(())
    }

	// Set simulation grid cell size.
    pub fn set_grid_cell_size(
        &mut self,
        cell_size: u16) -> Result<()> {

        self.cell_size = cell_size;

        Ok(())
    }

    pub fn get_velocity_point_pos(&self, row_index: usize, col_index: usize, horizontal: bool) -> Vec2 {
        // This function receives a row and column to index the point in either
        // `self.velocity_u` or `self.velocity_v` and find where their (x, y)
        // coords are.

        // Since the horizontal velocity points (u) have one more horizontally
        // and the vertical velocity points (v) have one more vertically,
        // the `horizontal` parameter is needed to differentiate between
        // `self.velocity_u` and `self.velocity_v`.

        let grid_height = self.dimensions.0 * self.cell_size;
        let grid_length = self.dimensions.1 * self.cell_size;

        let offset = (self.cell_size / 2) as f32;

        if horizontal {
            let pos_x = col_index as f32 * self.cell_size as f32;
            let pos_y = grid_height as f32 - (row_index as f32 * self.cell_size as f32 + offset);

            return Vec2::new(pos_x, pos_y);

        } else {
            let pos_x = col_index as f32 * self.cell_size as f32 + offset;
            let pos_y = grid_height as f32 - (row_index as f32 * self.cell_size as f32);

            return Vec2::new(pos_x, pos_y);
        }

    }

	/** Get the collision value of a cell; returns 0 if SimGridCellType::Solid OR if cell_x or
		cell_y are out of bounds.  Returns 1 if SimGridCellType::Fluid or SimGridCellType::Air. */
	pub fn get_cell_type_value(&self, cell_row: usize, cell_col: usize) -> u8 {

		// Because cell_x and cell_y are unsigned, we do not need an underflow check.
		if cell_row >= self.dimensions.0 as usize ||
			cell_col >= self.dimensions.1 as usize {
			return 0;
		}

		/* When modifying flow out of a cell, we need to modify said flow by 0 if the
			cell the flow is going into is solid.  If the cell is not solid, we leave flow
			unmodified. */
		match self.cell_type[cell_row][cell_col] {
			SimGridCellType::Solid	=> 0,
			SimGridCellType::Fluid	=> 1,
			SimGridCellType::Air	=> 1,
		}
	}

	/** Convert the Vec2 position (x, y) to coordinates (row, column).  **will return the
		closest valid cell to any invalid position input.** */
	pub fn get_cell_coordinates_from_position(&self, position: &Vec2) -> Vec2 {
		let cell_size: f32			= self.cell_size as f32;
		let grid_upper_bound: f32	= self.dimensions.1 as f32 * cell_size;

		let mut coordinates: Vec2 = Vec2 {
			x: f32::floor((grid_upper_bound - position[1]) / cell_size),	// Row
			y: f32::floor(position[0] / cell_size),							// Column
		};

		// Clamp our coordinates to our grid's bounds.
		coordinates[0] = f32::max(0.0, coordinates[0]);
		coordinates[1] = f32::max(0.0, coordinates[1]);
		coordinates[0] = f32::min((self.dimensions.0 - 1) as f32, coordinates[0]);
		coordinates[1] = f32::min((self.dimensions.1 - 1) as f32, coordinates[1]);

		coordinates
	}

	/** Convert the Vec2 coordinates (row, column) to a position (x, y).  **will return the
		closest valid position to any invalid coordinate input.** */
	pub fn get_cell_position_from_coordinates(&self, coordinates: Vec2) -> Vec2 {
		let cell_size: f32			= self.cell_size as f32;
		let grid_max_x_bound: f32	= self.dimensions.1 as f32 * cell_size;
		let grid_max_y_bound: f32	= self.dimensions.0 as f32 * cell_size - cell_size;

		let mut position: Vec2 = Vec2 {
			x: f32::floor(coordinates.y * cell_size),
			y: f32::floor(grid_max_y_bound - coordinates.x * cell_size),
		};

		// Clamp our coordinates to our grid's bounds.
		position.x = f32::max(0.0, position.x);
		position.y = f32::max(0.0, position.y);
		position.x = f32::min(grid_max_x_bound, position.x);
		position.y = f32::min(grid_max_y_bound, position.y);

		position
	}
	
	/// Find the center position of a cell given its coordinates.
	pub fn get_cell_center_position_from_coordinates(&self, coordinates: &Vec2) -> Vec2 {
		let half_cell_size: f32	= (self.cell_size as f32) / 2.0;
		let cell_x: f32			= coordinates.y * self.cell_size as f32;
		let cell_y: f32			= coordinates.x * self.cell_size as f32;
		let grid_height: f32	= (self.dimensions.0 * self.cell_size) as f32;
		
		let cell_center_position: Vec2 = Vec2 {
			x: cell_x + half_cell_size,
			y: grid_height - cell_y - half_cell_size,
		};
		cell_center_position
	}

	/** Selects grid cells that entirely cover the a circle of radius `radius` centered at `position`;
		returns a Vector containing each cell's coordinates.  Note: the returned vector is of a 
		static size.  If any cells in the selection are outside of the grid, then the closest valid 
		cells will be added into the result.  **This can result in duplicated cell values, which is 
		necessary to ensure accurate density calculations (corner cells would otherwise be 
		considered much less dense than cells with selections entirely contained within the grid).** */
	pub fn select_grid_cells(&self, position: Vec2, radius: f32) -> Vec<Vec2> {

		/* If we are less than a cell in radius, the function will only search 1 cell.  That is
			incorrect, as we could still need to search 4 cells if the selection is positioned
			properly.  Therefore, we cap the radius for selection-cell bound checking to 2.5, but
			leave the true radius untouched to retain proper particle selection behavior. */
		let min_selection_size: f32 = self.cell_size as f32 / 2.0;
		let adj_radius: f32			= f32::max(min_selection_size, radius);

		/* Find our min/max world coordinates for cells to search.  Add the cell size to account for 
			the selection area potentially not being perfectly centered; this will ensure we always
			check the full possible number of cells our selection may be concerned with. We may check
			one or two extra cells, but I believe consistent behavior is worth 4 extra cell checks. */
		let selection_max_bound: Vec2 = Vec2 {
			x: position.x + adj_radius + self.cell_size as f32,
			y: position.y + adj_radius + self.cell_size as f32,
		};
		let selection_min_bound: Vec2 = Vec2 {
			x: position.x - adj_radius,
			y: position.y - adj_radius,
		};

		// Find the number of cells we need to check.
		let mut x_cell_count: usize			= (selection_max_bound.x - selection_min_bound.x) as usize;
		let mut y_cell_count: usize			= (selection_max_bound.y - selection_min_bound.y) as usize;
		x_cell_count						/= self.cell_size as usize;
		y_cell_count						/= self.cell_size as usize;
		let cells_in_selection_count: usize	= x_cell_count * y_cell_count;

		// Figure out which grid cells we are actually going to be checking.
		let mut cells_in_selection: Vec<Vec2>	= vec![Vec2::ZERO; cells_in_selection_count];
		for cell_index in 0..cells_in_selection_count {

			/* BUG: Sometimes the top two corner cells of the selection "flicker", and the sides have
				an extra cell jutting out.  Not sure why, but my guess is it's a type casting or
				rounding issue; not important (for now).  The corner flickering does affect the number
				of cells checked, however the extra cell jutting out does not (making me think the
				latter is a rendering issue).  Finally, the algorithm breaks down a little bit extra
				if the radius is not a multiple of the grid cell size. */
			
			// Per cell, get the x and y indices through our cell selection array.
			let cell_y_index: usize	= (cell_index / y_cell_count) % y_cell_count;
			let cell_x_index: usize	= cell_index % x_cell_count;
			
			// Convert the cell's x and y indices into a position, and then into a grid coordinate.
			let cell_position: Vec2 = Vec2 {
				x: selection_min_bound.x + cell_x_index as f32 * self.cell_size as f32,
				y: selection_min_bound.y + cell_y_index as f32 * self.cell_size as f32
			};
			
			let cell_coordinates = self.get_cell_coordinates_from_position(&cell_position);

			// Add our selected cell's coordinates to our list of cell coordinates!
			cells_in_selection[cell_index] = cell_coordinates;
		}

		cells_in_selection
	}

	/// Set all density values within the grid to 0.0.
	pub fn clear_density_values(&mut self) {
		for density in self.density.iter_mut() {
			*density = 0.0;
		}
	}

	/// Update each grid cell's density based on weighted particle influences.
	pub fn update_grid_density(&mut self, particle_position: Vec2) {

		/* Select all 9 nearby cells so we can weight their densities; a radius of 0.0 
			automatically clamps to a 3x3 grid of cells surrounding the position vector. */
		let nearby_cells = self.select_grid_cells(particle_position, 0.0);
		
		// For each nearby cell, add weighted density value based on distance to particle_position.
		for cell in nearby_cells.iter() {
			let cell_lookup_index = get_lookup_index(*cell, self.dimensions.0);
			
			// Get the center of the cell so we can weight density properly.
			let cell_position: Vec2		= self.get_cell_position_from_coordinates(*cell);
			let cell_center: Vec2		= Vec2 {
				x: cell_position.x,
				y: cell_position.y
			};
			
			// Distance squared to save ourselves the sqrt(); density is arbitrary here anyways.
			self.density[cell_lookup_index] += cell_center.distance_squared(particle_position);
		}
	}
	
	/// Gets an interpolated density value for a lookup index within the grid's bounds.
	pub fn get_density_at_position(&self, position: Vec2) -> f32 {
		
		let mut density: f32 = 0.0;
		
		// Select all 9 nearby cells so we can query their densities.
		let nearby_cells = self.select_grid_cells(position, self.cell_size as f32);
		
		// For each nearby cell, add its density weighted based on position to final density value.
		for cell in nearby_cells.iter() {
			
			// Get the center of the cell so we can weight density properly.
			let cell_position: Vec2		= self.get_cell_position_from_coordinates(*cell);
			let cell_center: Vec2		= Vec2 {
				x: cell_position.x + (0.5 * self.cell_size as f32),
				y: cell_position.y - (0.5 * self.cell_size as f32)
			};
			
			let cell_lookup_index = get_lookup_index(*cell, self.dimensions.0);
			density += self.density[cell_lookup_index];
		}
		
		density
	}

	/// Add a new particle into our spatial lookup table.
	pub fn add_particle_to_lookup(&mut self, particle_id: Entity, lookup_index: usize) {

		if lookup_index > self.spatial_lookup.len() {
			eprintln!("Particle lookup index is out-of-bounds; cannot add particle to table!");
			return;
		}
		self.spatial_lookup[lookup_index].push(particle_id);
	}

	/// Remove a particle from our spatial lookup table; does nothing if the particle isn't found.
	pub fn remove_particle_from_lookup(&mut self, particle_id: Entity, lookup_index: usize) {

		if lookup_index > self.spatial_lookup.len() {
			eprintln!("Particle lookup index is out-of-bounds; cannot remove particle from table!");
			return;
		}

		// Search through our spatial lookup at the specified location.
		for particle_index in 0..self.spatial_lookup[lookup_index].len() {

			// If we found it, remove it.
			if self.spatial_lookup[lookup_index][particle_index] == particle_id {
				self.spatial_lookup[lookup_index].swap_remove(particle_index);
				break;
			}
		}
	}

	/// Get a Vec<Entity> of the particles currently inside of the cell at lookup_index.
	pub fn get_particles_in_lookup(&self, lookup_index: usize) -> Vec<Entity> {

		// Return an empty vector if we are out of bounds.
		if lookup_index >= (self.dimensions.0 * self.dimensions.1) as usize {
			return Vec::new();
		}

		let mut lookup_vector: Vec<Entity> = Vec::new();

		for particle_id in self.spatial_lookup[lookup_index].iter() {

			// TODO: Don't use placeholder!  Bad kitty!!!
			if *particle_id == Entity::PLACEHOLDER {
				continue;
			}

			lookup_vector.push(*particle_id);
		}

		lookup_vector
	}

    /// Get velocity of the cell
    pub fn get_cell_velocity(&self, row: usize, column: usize) -> Vec2 {

        if row as u16 >= self.dimensions.0 || column as u16 >= self.dimensions.1 || row == 0 || column == 0 {
            return Vec2::ZERO;
        }

        let left_u = self.velocity_u[row][column];
        let right_u = self.velocity_u[row][column + 1];
        let top_v = self.velocity_v[row][column];
        let down_v = self.velocity_v[row + 1][column];

        let u_avg = (left_u + right_u) / 2.0;
        let v_avg = (top_v + down_v) / 2.0;

        let velocity = Vec2::new(u_avg, v_avg);

        velocity

    }

	/// Get the particles in all 9 cells surrounding a point.
	fn get_nearby_particles(&self, lookup_index: usize) -> Vec<Entity> {

		let mut nearby_particles: Vec<Entity>	= Vec::new();
		let mut cells_to_check: Vec<usize>		= Vec::new();
		let col_count: usize					= self.dimensions.1 as usize;

		let is_cell_on_right_border: bool		= lookup_index % (col_count - 1) == 0;
		let is_cell_on_left_border: bool		= lookup_index % col_count == 0;

		/* Make sure the current row's cells-to-check are valid.  If they are, search for particles
			within them. */
		cells_to_check.push(lookup_index);
		if lookup_index > 0 && !is_cell_on_left_border {
			cells_to_check.push(lookup_index - 1);
		}
		if lookup_index < self.spatial_lookup.len() && !is_cell_on_right_border {
			cells_to_check.push(lookup_index + 1);
		}

		// Previous row's cell check:
		if lookup_index >= col_count {
			cells_to_check.push(lookup_index - col_count);
			if !is_cell_on_left_border {
				cells_to_check.push(lookup_index - col_count - 1);
			}
			if !is_cell_on_right_border {
				cells_to_check.push(lookup_index - col_count + 1);
			}
		}

		// Next row's cell check:
		if lookup_index <= self.spatial_lookup.len() - col_count {
			cells_to_check.push(lookup_index + col_count);
			if !is_cell_on_left_border {
				cells_to_check.push(lookup_index + col_count - 1);
			}
			if lookup_index < self.spatial_lookup.len() - col_count && !is_cell_on_right_border {
				cells_to_check.push(lookup_index + col_count + 1);
			}
		}


		for i in 0..cells_to_check.len() {
			nearby_particles.append(&mut self.get_particles_in_lookup(cells_to_check[i]));
		}

		nearby_particles
	}
}

#[derive(Component, Debug)]
pub struct SimParticle {
	pub position:		Vec2, 	// This particle's [x, y] position.
	pub velocity:		Vec2, 	// This particle's [x, y] velocity.
	pub lookup_index:	usize,	// Bucket index into spatial lookup for efficient neighbor search.
}
