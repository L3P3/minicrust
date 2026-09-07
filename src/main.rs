#![windows_subsystem = "windows"]

use std::rc::Rc;

use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::window::Window;

mod constants;
mod input;
mod player;
mod renderer;
mod world;

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().unwrap();
	let mut window: Option<Rc<Window>> = None;
	let mut renderer: Option<renderer::Renderer> = None;

	let world = world::World::new();
	let mut player = player::Player::new(world);
	let time_start = std::time::Instant::now();

	#[allow(deprecated)]
	event_loop.run(move |event, event_loop| {
		event_loop.set_control_flow(ControlFlow::Wait);

		let mut render_try = |player: &player::Player| {
			if let (Some(window), Some(renderer)) = (&window, &mut renderer) {
				renderer.frame_render(window, player, time_start.elapsed());
			}
		};

		match event {
			/* Event::AboutToWait => {
				if let Some(window) = &window {
					window.request_redraw();
				}
			}, */
			Event::Resumed => {
				if window.is_none() {
					let new_window = Rc::new(
						event_loop.create_window(
							Window::default_attributes()
								.with_title(format!("minicrust {}", env!("CARGO_PKG_VERSION"))),
						)
						.unwrap(),
					);
					let context = softbuffer::Context::new(new_window.clone()).unwrap();
					let surface = softbuffer::Surface::new(&context, new_window.clone()).unwrap();
					renderer = Some(renderer::Renderer::new(surface));
					window = Some(new_window);
				}
			},
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => {
					event_loop.exit();
				},
				WindowEvent::KeyboardInput { event, .. } => {
					input::key_input(&event, &mut player);
					render_try(&player);
				},
				WindowEvent::RedrawRequested {} => {
					render_try(&player);
				},
				_ => {},
			},
			_ => {},
		}
	}).unwrap();
}
