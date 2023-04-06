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
	framebuffer: framebuffer::Framebuffer,
	pixels: &'static mut [Pixel],
}

impl Renderer {
	pub fn new() -> Self {
		let mut framebuffer = framebuffer::Framebuffer::new("/dev/fb0")
			.expect("cannot open framebuffer device");

		// some toasters might differ
		assert!(framebuffer.var_screen_info.bits_per_pixel == 32, "32 bpp only");

		// hack our own mutable slice to prevent copying every frame
		let pixels = unsafe {
			std::slice::from_raw_parts_mut(
				framebuffer.frame.as_mut_ptr() as *mut Pixel,
				framebuffer.frame.len() / std::mem::size_of::<Pixel>(),
			)
		};

		framebuffer::Framebuffer::set_kd_mode(framebuffer::KdMode::Graphics).unwrap();

		Self {
			framebuffer,
			pixels,
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
		time: f32,
	) {
		let resolution_x = self.framebuffer.var_screen_info.xres as usize;
		//let resolution_y = self.framebuffer.var_screen_info.yres as usize;

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
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {
		framebuffer::Framebuffer::set_kd_mode(framebuffer::KdMode::Text).unwrap();
	}
}
