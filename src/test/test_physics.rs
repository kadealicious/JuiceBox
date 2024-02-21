use bevy::math::Vec2;
use bevy::prelude::*;
use crate::simulation::sim_physics_engine::particles_to_grid;
use crate::simulation::util::interpolate_velocity;
use crate::simulation::{self, SimConstraints, SimGrid, SimParticle};

#[test]
fn interpolation_test() {
   let particle_pos = Vec2::new(12.0, 25.0);

   let mut grid = SimGrid::default();

   for row in 0..(grid.dimensions.1 + 1) as usize {
       for col in 0..grid.dimensions.0 as usize {
           grid.velocity_v[row][col] = -9.8;
       }
   }

   let goal = Vec2::new(0.0, -9.8);

   let interpolated_velocity = interpolate_velocity(particle_pos, &grid);

   let within_reason = (interpolated_velocity.y - goal.y).abs() < 0.001;

   assert_eq!(within_reason, true);
}

#[test]
fn velocity_transfer_test() {

    let mut juicebox_test = App::new();

    juicebox_test.insert_resource(SimGrid::default());
    juicebox_test.insert_resource(SimConstraints::default());

	juicebox_test.add_systems(Startup, simulation::test_setup);
	juicebox_test.add_systems(Update, simulation::test_update);

    juicebox_test.update();

    let grid = juicebox_test.world.resource::<SimGrid>();

    let vel_u = grid.velocity_u.clone();
    let vel_v =  grid.velocity_v.clone();

    let mut transfer_u = false;
    let mut transfer_v = false;
    let mut transfer_particle = false;

    let (rows, cols) = grid.dimensions;

    for row in 0..rows as usize {
        for col in 0..cols as usize + 1 {
            if vel_u[row][col] != f32::MIN {
                transfer_u = true;
            }
        }
    }

    for row in 0..rows as usize + 1 {
        for col in 0..cols as usize {
            if vel_v[row][col] != f32::MIN {
                transfer_v = true;
            }
        }
    }

    for particle in juicebox_test.world.query::<&SimParticle>().iter(&juicebox_test.world) {
        if particle.velocity != Vec2::ZERO {
            transfer_particle = true;
        }
    }

    assert_eq!(true, transfer_u);
    assert_eq!(true, transfer_v);
    assert_eq!(true, transfer_particle);
}

#[test]
fn extrapolate_test() {

    let mut juicebox_test = App::new();

    juicebox_test.insert_resource(SimGrid::default());
    juicebox_test.insert_resource(SimConstraints::default());

	juicebox_test.add_systems(Startup, simulation::test_setup);
	juicebox_test.add_systems(Update, simulation::test_update);

    juicebox_test.update();

    let mut success = true;

    let grid = juicebox_test.world.resource::<SimGrid>().clone();

    for particle in juicebox_test.world.query::<&SimParticle>().iter(&juicebox_test.world) {

        let particle_coords = grid.get_cell_coordinates_from_position(&particle.position);

        let offsets: [[i32; 2]; 4] = [
            [0, 2],
            [0, -2],
            [2, 0],
            [-2, 0],
        ];

        for offset in offsets {
            let cell_vel = grid.get_cell_velocity((particle_coords.x as i32 + offset[0]) as usize, (particle_coords.y as i32 + offset[1]) as usize);

            println!("{:?}", cell_vel);

            if cell_vel.x.abs() == f32::INFINITY || cell_vel.y.abs() == f32::INFINITY {
                success = false;
            }
        }

    }

    assert_eq!(true, success);
}
