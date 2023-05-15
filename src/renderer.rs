use rayon::prelude::*;

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Pixel {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

pub struct Renderer {
	graphics_context: softbuffer::GraphicsContext,
	pixels: Vec<Pixel>,
}

impl Renderer {
	pub fn new(
		graphics_context: softbuffer::GraphicsContext,
	) -> Self {
		Self {
			graphics_context,
			pixels: vec![],
		}
	}

	fn pixel_render(x: usize, y: usize) -> Pixel {
		// TODO
		Pixel {
			r: (x & 0xff) as u8,
			g: (y & 0xff) as u8,
			b: 0x00,
			a: 0xff,
		}
	}

	pub fn frame_render(
		&mut self,
		window: &winit::window::Window,
		time: f32,
	) {
		let inner_size = window.inner_size();
		let resolution_x = inner_size.width as usize;
		let resolution_y = inner_size.height as usize;

		self.pixels.resize(resolution_x * resolution_y, Pixel {
			r: 0x00,
			g: 0x00,
			b: 0x00,
			a: 0xff,
		});

		// move the pixels around (looks funny)
		let offset_x = (time.sin() * 50.0 + 50.0).round() as usize;
		let offset_y = (time.cos() * 50.0 + 50.0).round() as usize;

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
