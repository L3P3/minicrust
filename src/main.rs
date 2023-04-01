extern crate ctrlc;
extern crate framebuffer;
extern crate rayon;

mod constants;
mod renderer;
mod world;

fn main() {
	// handle ctrl+c, the fancy rust way
	let running = std::sync::Arc::new(
		std::sync::atomic::AtomicBool::new(true)
	);
	ctrlc::set_handler({
		let running = running.clone();
		move || {
			running.store(false, std::sync::atomic::Ordering::Relaxed);
		}
	}).unwrap();

	let mut renderer = renderer::Renderer::new();
	let _world = world::World::new();
	let time_start = std::time::Instant::now();

	while running.load(std::sync::atomic::Ordering::Relaxed) {
		let time = time_start.elapsed().as_secs_f32();

		renderer.frame_render(time);

		// TODO: use vsync
		std::thread::sleep(std::time::Duration::from_millis(16));
	}
}
