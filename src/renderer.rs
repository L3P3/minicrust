use std::rc::Rc;

use num_integer::Roots;
use rayon::prelude::*;
use winit::window::Window;

use crate::constants::*;
use crate::player;

type SoftSurface = softbuffer::Surface<Rc<Window>, Rc<Window>>;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
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

}

const DIMENSION_Y: u8 = 0;
const DIMENSION_X: u8 = 1;
const DIMENSION_Z: u8 = 2;
const STEP_RAW_MIN: f32 = 1.0 / 64.0;

const COLOR_BLACK: Pixel = Pixel::rgb(0x00, 0x00, 0x00);
const COLOR_SKY: Pixel = Pixel::rgb(0x84, 0xb1, 0xff);
const COLOR_HIT_Y: Pixel = Pixel::rgb(0x00, 0xff, 0x00);
const COLOR_HIT_X: Pixel = Pixel::rgb(0xff, 0x00, 0x00);
const COLOR_HIT_Z: Pixel = Pixel::rgb(0x00, 0x00, 0xff);

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
		let fov = 80.0_f32 / 45.0_f32; // TODO
		let view_distance: u32 = 10 << 8;

		let fov_step = fov / resolution_x.max(resolution_y) as f32;
		let angle_h = player.angle_h;
		let angle_v = player.angle_v;
		let position_y: i16 = player.position_y;
		let position_x: i32 = player.position_x;
		let position_z: i32 = player.position_z;
		// view angle vectors
		let angle_v_vec = (angle_v.sin(), angle_v.cos());
		let angle_h_vec = (angle_h.sin(), angle_h.cos());

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
			let step_y_raw = canvas_y_relative * angle_v_vec.1 - angle_v_vec.0;
			let angle_v = canvas_y_relative * angle_v_vec.0 + angle_v_vec.1;
			let step_x_center = angle_v * angle_h_vec.0;
			let step_z_center = angle_v * angle_h_vec.1;
			let step_y_primary: i8 = 1 - 2 * ((step_y_raw < 0.0) as i8);
			let step_y_inverse = 1.0 / step_y_raw.abs();
			let mut dimension_next: u8 = DIMENSION_X;

			// render row pixels sequentially
			for (canvas_x, pixel) in line.iter_mut().enumerate() {
				let canvas_x_relative = (canvas_x as f32 - resolution_x_h) * fov_step;
				let step_x_raw = step_x_center + canvas_x_relative * angle_h_vec.1;
				let step_z_raw = step_z_center - canvas_x_relative * angle_h_vec.0;
				let dimension_offset = dimension_next;

				// black/blue skybox
				*pixel = if step_y_primary < 0 { COLOR_BLACK } else { COLOR_SKY };

				let mut check_distance_min: u32 = view_distance;

				for dimension in 0..3u8 {
					match (dimension + dimension_offset) % 3 {
						DIMENSION_Y => {
							if step_y_raw.abs() < STEP_RAW_MIN {
								continue;
							}

							let step_x: i32 = (step_x_raw * step_y_inverse * 256.0).round() as i32;
							let step_z: i32 = (step_z_raw * step_y_inverse * 256.0).round() as i32;

							let step_diagonal: u32 = (
								(step_x * step_x) as u32 +
								(step_z * step_z) as u32 +
								256u32 * 256u32
							).sqrt();

							// start position
							let offset: u8 = (position_y as u8) ^ ((step_y_primary > 0) as u8).wrapping_neg();
							let mut check_distance: u32 = (step_diagonal * offset as u32) >> 8;
							if check_distance >= check_distance_min {
								continue;
							}
							let mut check_y = (position_y >> 8) as i8;
							let mut check_x: i32 = position_x + ((step_x * offset as i32) >> 8);
							let mut check_z: i32 = position_z + ((step_z * offset as i32) >> 8);

							// add steps until collision or out of range
							loop {
								// move on
								check_y += step_y_primary;

								// check if inside world
								match
									(step_y_primary as u8) & 0b100 | // ((step_y_primary < 0) as u8) << 2
									((check_y >= CHUNK_HEIGHT as i8) as u8) << 1 |
									(check_y as u8) >> 7 // ((check_y < 0) as u8)
								{
									// will never reach a block
									0b010 | // step_y > 0.0 && check_y >= CHUNK_HEIGHT
									0b101 => break, // step_y < 0.0 && check_y < 0.0

									// will maybe reach a block later
									0b001 | // step_y > 0.0 && check_y < 0.0
									0b110 => {}, // step_y < 0.0 && check_y >= CHUNK_HEIGHT

									// inside world
									0b000 | 0b100 => {
										let block = world.block_get(
											(check_x >> 8) as u16,
											check_y as u8,
											(check_z >> 8) as u16,
										);

										if block != BlockType::Air {
											// collision
											*pixel = COLOR_HIT_Y;
											check_distance_min = check_distance;
											dimension_next = DIMENSION_Y;
											break;
										}
									},

									0b011 | 0b111 => unreachable!("cannot be above and below world at the same time"),

									_ => unreachable!("max 3 bits"),
								}

								// no collision yet, move on
								check_x += step_x;
								check_z += step_z;
								check_distance += step_diagonal;

								if check_distance >= check_distance_min {
									break;
								}
							}
						},

						DIMENSION_X => {
							if step_x_raw.abs() < STEP_RAW_MIN {
								continue;
							}

							let step_x_inverse = 1.0_f32 / step_x_raw.abs();
							let step_y: i32 = (step_y_raw * step_x_inverse * 256.0).round() as i32;
							let step_z: i32 = (step_z_raw * step_x_inverse * 256.0).round() as i32;

							let step_diagonal: u32 = (
								(step_y * step_y) as u32 +
								(step_z * step_z) as u32 +
								256u32 * 256u32
							).sqrt();

							let step_x: i16 = 1 - 2 * ((step_x_raw < 0.0) as i16);

							// start position
							let offset: u8 = (position_x as u8) ^ ((step_x_raw > 0.0) as u8).wrapping_neg();
							let mut check_distance: u32 = (step_diagonal * offset as u32) >> 8;
							if check_distance >= check_distance_min {
								continue;
							}
							let mut check_x = (position_x >> 8) as i16;
							let mut check_y: i32 = position_y as i32 + ((step_y * offset as i32) >> 8);
							let mut check_z: i32 = position_z + ((step_z * offset as i32) >> 8);

							// add steps until collision or out of range
							loop {
								// move on
								check_x += step_x;

								// check if inside world
								match
									((step_y < 0) as u8) << 2 |
									((check_y >= (CHUNK_HEIGHT as i32) << 8) as u8) << 1 |
									((check_y < 0) as u8)
								{
									// will never reach a block
									0b010 | // step_y > 0.0 && check_y >= CHUNK_HEIGHT
									0b101 => break, // step_y < 0.0 && check_y < 0.0

									// will maybe reach a block later
									0b001 | // step_y > 0.0 && check_y < 0.0
									0b110 => {}, // step_y < 0.0 && check_y >= CHUNK_HEIGHT

									// inside world
									0b000 | 0b100 => {
										let block = world.block_get(
											check_x as u16,
											(check_y >> 8) as u8,
											(check_z >> 8) as u16,
										);

										if block != BlockType::Air {
											// collision
											*pixel = COLOR_HIT_X;
											check_distance_min = check_distance;
											dimension_next = DIMENSION_X;
											break;
										}
									},

									0b011 | 0b111 => unreachable!("cannot be above and below world at the same time"),

									_ => unreachable!("max 3 bits"),
								}

								// no collision yet, move on
								check_y += step_y;
								check_z += step_z;
								check_distance += step_diagonal;

								if check_distance >= check_distance_min {
									break;
								}
							}
						},

						DIMENSION_Z => {
							if step_z_raw.abs() < STEP_RAW_MIN {
								continue;
							}

							let step_z_inverse = 1.0_f32 / step_z_raw.abs();
							let step_x: i32 = (step_x_raw * step_z_inverse * 256.0).round() as i32;
							let step_y: i32 = (step_y_raw * step_z_inverse * 256.0).round() as i32;

							let step_diagonal: u32 = (
								(step_x * step_x) as u32 +
								(step_y * step_y) as u32 +
								256u32 * 256u32
							).sqrt();

							let step_z: i16 = 1 - 2 * ((step_z_raw < 0.0) as i16);

							// start position
							let offset: u8 = (position_z as u8) ^ ((step_z_raw > 0.0) as u8).wrapping_neg();
							let mut check_distance: u32 = (step_diagonal * offset as u32) >> 8;
							if check_distance >= check_distance_min {
								continue;
							}
							let mut check_x: i32 = position_x + ((step_x * offset as i32) >> 8);
							let mut check_y: i32 = position_y as i32 + ((step_y * offset as i32) >> 8);
							let mut check_z = (position_z >> 8) as i16;

							// add steps until collision or out of range
							loop {
								// move on
								check_z += step_z;

								// check if inside world
								match
									((step_y < 0) as u8) << 2 |
									((check_y >= (CHUNK_HEIGHT as i32) << 8) as u8) << 1 |
									((check_y < 0) as u8)
								{
									// will never reach a block
									0b010 | // step_y > 0.0 && check_y >= CHUNK_HEIGHT
									0b101 => break, // step_y < 0.0 && check_y < 0.0

									// will maybe reach a block later
									0b001 | // step_y > 0.0 && check_y < 0.0
									0b110 => {}, // step_y < 0.0 && check_y >= CHUNK_HEIGHT

									// inside world
									0b000 | 0b100 => {
										let block = world.block_get(
											(check_x >> 8) as u16,
											(check_y >> 8) as u8,
											check_z as u16,
										);

										if block != BlockType::Air {
											// collision
											*pixel = COLOR_HIT_Z;
											check_distance_min = check_distance;
											dimension_next = DIMENSION_Z;
											break;
										}
									},

									0b011 | 0b111 => unreachable!("cannot be above and below world at the same time"),

									_ => unreachable!("max 3 bits"),
								}

								// no collision yet, move on
								check_x += step_x;
								check_y += step_y;
								check_distance += step_diagonal;

								if check_distance >= check_distance_min {
									break;
								}
							}
						},

						_ => unreachable!("only 3 dimensions"),
					}
				}
			}
		});

		buffer.present().unwrap();
	}
}
