use crate::world;

pub struct Player {
	pub angle_h: f32,
	pub angle_v: f32,
	/// x position in q24.8
	pub position_x: i32,
	/// y position in q8.8
	pub position_y: i16,
	/// z position in q24.8
	pub position_z: i32,
	pub world: world::World,
}

impl Player {
	pub fn new(world: world::World) -> Self {
		Self {
			angle_h: 0.0_f32,
			angle_v: 0.0_f32,
			position_x: world.spawn_x,
			position_y: world.spawn_y,
			position_z: world.spawn_z,
			world,
		}
	}
}
