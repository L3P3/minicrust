use rayon::prelude::*;

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Pixel {
	b: u8,
	g: u8,
	r: u8,
	a: u8,
}

pub struct Renderer {
	framerate_age: std::time::Duration,
	framerate_counter: u32,
	graphics_context: softbuffer::GraphicsContext,
	pixels: Vec<Pixel>,
	time_last: std::time::Duration,
}

impl Renderer {
	pub fn new(
		graphics_context: softbuffer::GraphicsContext,
	) -> Self {
		Self {
			framerate_age: std::time::Duration::new(0, 0),
			framerate_counter: 0,
			graphics_context,
			pixels: vec![],
			time_last: std::time::Duration::new(0, 0),
		}
	}

	fn pixel_render(x: usize, y: usize) -> Pixel {
		// TODO
		Pixel {
			r: (x & 0xff) as u8,
			g: (y & 0xff) as u8,
			b: 0xff,
			a: 0x00,
		}
	}

	pub fn frame_render(
		&mut self,
		window: &winit::window::Window,
		time: std::time::Duration,
	) {
		let time_delta = time - self.time_last;
		self.time_last = time;

		// framerate counter, title
		self.framerate_age += time_delta;
		self.framerate_counter += 1;
		if self.framerate_age.as_millis() >= 1000 {
			let title = format!(
				"Minicrust {} — {} fps",
				env!("CARGO_PKG_VERSION"),
				self.framerate_counter,
			);
			window.set_title(&title);
			self.framerate_age = std::time::Duration::new(0, 0);
			self.framerate_counter = 0;
		}

		// screen size
		let inner_size = window.inner_size();
		let resolution_x = inner_size.width as usize;
		let resolution_y = inner_size.height as usize;

		self.pixels.resize(resolution_x * resolution_y, Pixel {
			r: 0x00,
			g: 0x00,
			b: 0x00,
			a: 0x00,
		});

		// move the pixels around (looks funny)
		let time_f32 = time.as_secs_f32();
		let offset_x = (time_f32.sin() * 50.0 + 50.0).round() as usize;
		let offset_y = (time_f32.cos() * 50.0 + 50.0).round() as usize;

		// render rows in parallel
		self.pixels.par_chunks_exact_mut(resolution_x).enumerate().for_each(|(y, line)| {
			for (x, pixel) in line.iter_mut().enumerate() {
				let x = x + offset_x;
				let y = y + offset_y;
				*pixel = Self::pixel_render(x, y);
			}
		});

		let buffer = unsafe {
			std::slice::from_raw_parts(
				self.pixels.as_ptr() as *const u32,
				self.pixels.len(),
			)
		};

		self.graphics_context.set_buffer(
			buffer,
			resolution_x as u16,
			resolution_y as u16,
		);
	}
}
