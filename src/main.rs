extern crate rayon;
extern crate softbuffer;
extern crate winit;

mod constants;
mod renderer;
mod world;

fn main() {
	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_title(format!("Minicrust {}", env!("CARGO_PKG_VERSION")))
		.build(&event_loop)
		.unwrap();
	let graphics_context = unsafe {
		softbuffer::GraphicsContext::new(&window, &window)
	}.unwrap();

	let mut renderer = renderer::Renderer::new(graphics_context);
	let _world = world::World::new();
	let time_start = std::time::Instant::now();

	event_loop.run(move |event, _, control_flow| {
		control_flow.set_poll();

		match event {
			winit::event::Event::WindowEvent {
				event,
				..
			} => match event {
				winit::event::WindowEvent::CloseRequested => {
					control_flow.set_exit();
				},
				_ => {},
			},
			winit::event::Event::MainEventsCleared => {
				window.request_redraw();
			},
			winit::event::Event::RedrawRequested(_) => {
				renderer.frame_render(&window, time_start.elapsed());
			},
			_ => {},
		}
	});
}
