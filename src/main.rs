extern crate ctrlc;
extern crate framebuffer;
extern crate rayon;

use rayon::prelude::*;

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Pixel {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

fn pixel_render(x: i32, y: i32) -> Pixel {
	// TODO
	Pixel {
		r: (x % 256) as u8,
		g: (y % 256) as u8,
		b: 0,
		a: 255,
	}
}

fn frame_render(
	time_start: std::time::Instant,
	pixels: &mut [Pixel],
	xres: i32,
	_yres: i32,
) {
	// move the pixels around (looks funny)
	let time = time_start.elapsed().as_secs_f32();
	let offset_x = (time.sin() * 50.0 + 25.0).round() as i32;
	let offset_y = (time.cos() * 50.0 + 25.0).round() as i32;

	// use rayon to use all cpu threads
	pixels.par_iter_mut()
		.enumerate()
		.for_each(|(index, pixel)| {
			let index = index as i32;
			let x = index % xres + offset_x;
			let y = index / xres + offset_y;
			*pixel = pixel_render(x, y);
		});
}

fn main() {
	// handle ctrl+c, the fancy rust way
	let running = std::sync::Arc::new(
		std::sync::atomic::AtomicBool::new(true)
	);
	{
		let r = running.clone();
		ctrlc::set_handler(move || {
			r.store(false, std::sync::atomic::Ordering::Relaxed);
		}).unwrap();
	}

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

	let time_start = std::time::Instant::now();
	let xres = framebuffer.var_screen_info.xres as i32;
	let yres = framebuffer.var_screen_info.yres as i32;

	while running.load(std::sync::atomic::Ordering::Relaxed) {
		frame_render(
			time_start,
			pixels,
			xres,
			yres,
		);

		std::thread::sleep(std::time::Duration::from_millis(16));
	}

	framebuffer::Framebuffer::set_kd_mode(framebuffer::KdMode::Text).unwrap();
}
