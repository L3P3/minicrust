use std::rc::Rc;

use rayon::prelude::*;
use winit::window::Window;

use crate::constants::*;
use crate::player;

type SoftSurface = softbuffer::Surface<Rc<Window>, Rc<Window>>;

#[repr(C)]
#[derive(Clone, Copy)]
struct Pixel {
	b: u8,
	g: u8,
	r: u8,
	a: u8,
}

impl Pixel {
	#[inline(always)]
	const fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b, a: 0x00 }
	}

	fn scale(&self, factor: f32) -> Self {
		Self {
			r: (self.r as f32 * factor).round() as u8,
			g: (self.g as f32 * factor).round() as u8,
			b: (self.b as f32 * factor).round() as u8,
			a: self.a,
		}
	}
}

#[derive(Clone, Copy)]
enum BlockFace {
	East,
	West,
	Top,
	Bottom,
	North,
	South,
}

const SETTINGS_FOV: f32 = 80.0;
const SETTINGS_VIEW_DISTANCE_L2: u32 = 8 + 4;

// the step values are [-1..1] in q22; 22 to fit into the f32's 23 bit mantissa
const STEP_SCALE_L2: u32 = 22;
const STEP_SCALE: f32 = (1 << STEP_SCALE_L2) as f32;

pub struct Renderer {
	framerate_age: std::time::Duration,
	framerate_counter: u32,
	surface: SoftSurface,
	time_last: std::time::Duration,
}

impl Renderer {
	pub fn new(
		surface: SoftSurface,
	) -> Self {
		Self {
			framerate_age: std::time::Duration::new(0, 0),
			framerate_counter: 0,
			surface,
			time_last: std::time::Duration::new(0, 0),
		}
	}

	/// update the framerate counter and title bar
	fn framerate_tick(
		&mut self,
		time_delta: std::time::Duration,
		window: &winit::window::Window,
	) {
		self.framerate_age += time_delta;
		self.framerate_counter += 1;
		if self.framerate_age.as_millis() >= 1000 {
			let title = format!(
				"minicrust {} — {} fps",
				env!("CARGO_PKG_VERSION"),
				self.framerate_counter,
			);
			window.set_title(&title);
			self.framerate_age = std::time::Duration::new(0, 0);
			self.framerate_counter = 0;
		}
	}

	pub fn frame_render(
		&mut self,
		window: &winit::window::Window,
		player: &player::Player,
		time: std::time::Duration,
	) {
		let world = &player.world;

		let time_delta = time - self.time_last;
		self.time_last = time;

		self.framerate_tick(time_delta, window);

		// screen size
		let inner_size = window.inner_size();
		let resolution_x = inner_size.width as usize;
		let resolution_y = inner_size.height as usize;
		assert!(resolution_x != 0 && resolution_y != 0, "no window area");
		let resolution_x_h = (resolution_x >> 1) as f32;
		let resolution_y_h = (resolution_y >> 1) as f32;

		self.surface
			.resize(
				unsafe { std::num::NonZeroU32::new_unchecked(resolution_x as u32) },
				unsafe { std::num::NonZeroU32::new_unchecked(resolution_y as u32) },
			)
			.unwrap();

		// TODO placeholders
		let fov = SETTINGS_FOV / 45.0; // TODO
		// pixel size on virtual screen
		let fov_step = fov / resolution_x.max(resolution_y) as f32;
		let position_x: i32 = player.position_x;
		let position_y: i32 = player.position_y as i32;
		let position_z: i32 = player.position_z;
		let (angle_v_sin, angle_v_cos) = player.angle_v.sin_cos();
		let (angle_h_sin, angle_h_cos) = player.angle_h.sin_cos();

		// zauber a pixel buffer from the surface
		let mut buffer = self.surface.buffer_mut().unwrap();
		let pixels: &mut [Pixel] = unsafe {
			std::slice::from_raw_parts_mut(
				buffer.as_mut_ptr().cast::<Pixel>(),
				buffer.len()
			)
		};
		// render rows in parallel
		pixels.par_chunks_exact_mut(resolution_x).enumerate().for_each(|(canvas_y, line)| {
			let canvas_y_relative = (resolution_y_h - canvas_y as f32) * fov_step;
			// vertical head rotation
			let step_y_row = canvas_y_relative * angle_v_cos - angle_v_sin;
			let step_xz_row = canvas_y_relative * angle_v_sin + angle_v_cos;
			// apply horizontal head rotation
			let step_x_center = step_xz_row * angle_h_sin;
			let step_z_center = step_xz_row * angle_h_cos;

			// render row pixels sequentially, using dda raymarching
			'pixels: for (canvas_x, pixel) in line.iter_mut().enumerate() {
				let canvas_x_relative = (canvas_x as f32 - resolution_x_h) * fov_step;
				// ray direction, yet unnormalized
				let step_x_raw = step_x_center + canvas_x_relative * angle_h_cos;
				let step_z_raw = step_z_center - canvas_x_relative * angle_h_sin;

				// normalize to fixed point
				let step_inverse = 1.0 / (
					step_x_raw * step_x_raw +
					step_y_row * step_y_row +
					step_z_raw * step_z_raw
				).sqrt();
				// direction, in [-1..1] in q22
				let step_x = (step_x_raw * step_inverse * STEP_SCALE).round() as i32;
				let step_y = (step_y_row * step_inverse * STEP_SCALE).round() as i32;
				let step_z = (step_z_raw * step_inverse * STEP_SCALE).round() as i32;

				// split signs and amounts
				let step_x_sign: i32 = (step_x > 0) as i32 - (step_x < 0) as i32;
				let step_y_sign: i32 = (step_y > 0) as i32 - (step_y < 0) as i32;
				let step_z_sign: i32 = (step_z > 0) as i32 - (step_z < 0) as i32;
				let step_x_abs = step_x.unsigned_abs() as i32;
				let step_y_abs = step_y.unsigned_abs() as i32;
				let step_z_abs = step_z.unsigned_abs() as i32;

				// current position, in full blocks world coordinates
				let mut check_x_block = position_x >> 8;
				let mut check_y_block = position_y >> 8;
				let mut check_z_block = position_z >> 8;

				// current position, in q8 relative to player position, starting at first block boundary
				let mut check_x_relative = if step_x_sign < 0 {
					position_x - (check_x_block << 8)
				} else {
					((check_x_block + step_x_sign) << 8) - position_x
				};
				let mut check_y_relative = if step_y_sign < 0 {
					position_y - (check_y_block << 8)
				} else {
					((check_y_block + step_y_sign) << 8) - position_y
				};
				let mut check_z_relative = if step_z_sign < 0 {
					position_z - (check_z_block << 8)
				} else {
					((check_z_block + step_z_sign) << 8) - position_z
				};

				'march: loop {
					// escaped world forever?
					if
						(check_y_block < 0 && step_y_sign <= 0) ||
						(check_y_block >= CHUNK_HEIGHT as i32 && step_y_sign >= 0)
					{
						break 'march;
					}

					let face_hit: BlockFace;

					// x boundary is closer than y boundary?
					if
						step_x_sign != 0 &&
						(
							step_y_sign == 0 ||
							(check_x_relative as i64 * step_y_abs as i64) <
							(check_y_relative as i64 * step_x_abs as i64)
						)
					{
						// x boundary is closer than z boundary?
						if
							step_z_sign == 0 ||
							(check_x_relative as i64 * step_z_abs as i64) <
							(check_z_relative as i64 * step_x_abs as i64)
						{
							if
								(check_x_relative as i64) << STEP_SCALE_L2 >
								(step_x_abs as i64) << SETTINGS_VIEW_DISTANCE_L2
							{
								break 'march;
							}

							check_x_block += step_x_sign;
							check_x_relative += 1 << 8;
							face_hit = if step_x_sign < 0 {
								BlockFace::West
							} else {
								BlockFace::East
							};
						}
						// z boundary is closer than x boundary?
						else {
							if
								(check_z_relative as i64) << STEP_SCALE_L2 >
								(step_z_abs as i64) << SETTINGS_VIEW_DISTANCE_L2
							{
								break 'march;
							}

							check_z_block += step_z_sign;
							check_z_relative += 1 << 8;
							face_hit = if step_z_sign < 0 {
								BlockFace::North
							} else {
								BlockFace::South
							};
						}
					}
					// y boundary is closer than x boundary?
					else {
						// y boundary is closer than z boundary?
						if
							step_y_sign != 0 &&
							(
								step_z_sign == 0 ||
								(check_y_relative as i64 * step_z_abs as i64) <
								(check_z_relative as i64 * step_y_abs as i64)
							)
						{
							if
								(check_y_relative as i64) << STEP_SCALE_L2 >
								(step_y_abs as i64) << SETTINGS_VIEW_DISTANCE_L2
							{
								break 'march;
							}

							check_y_block += step_y_sign;
							check_y_relative += 1 << 8;
							face_hit = if step_y_sign < 0 {
								BlockFace::Top
							} else {
								BlockFace::Bottom
							};
						}
						// z boundary is closer than y boundary?
						else {
							if
								step_z_sign == 0 ||
								(check_z_relative as i64) << STEP_SCALE_L2 >
								(step_z_abs as i64) << SETTINGS_VIEW_DISTANCE_L2
							{
								break 'march;
							}

							check_z_block += step_z_sign;
							check_z_relative += 1 << 8;
							face_hit = if step_z_sign < 0 {
								BlockFace::North
							} else {
								BlockFace::South
							};
						}
					}

					// inside world?
					if
						check_y_block >= 0 &&
						check_y_block < CHUNK_HEIGHT as i32
					{
						let block = world.block_get(
							check_x_block,
							check_y_block as u8,
							check_z_block
						);
						if block != BlockType::Air {
							let texture = Pixel::rgb(0x81, 0x5d, 0x42);
							*pixel = match face_hit {
								BlockFace::West | BlockFace::East => texture.scale(0.8),
								BlockFace::Top => texture,
								BlockFace::Bottom => texture.scale(0.4),
								BlockFace::North | BlockFace::South => texture.scale(0.6),
							};
							continue 'pixels;
						}
					}
				}

				// no hit, render skybox
				*pixel = if step_y_row < 0.0 {
					Pixel::rgb(0x00, 0x00, 0x00)
				} else {
					Pixel::rgb(0x84, 0xb1, 0xff)
				};
			}
		});

		buffer.present().unwrap();
	}
}
