use crate::player;

pub fn key_input(
	event: winit::event::KeyboardInput,
	player: &mut player::Player,
) {
	if let winit::event::ElementState::Pressed = event.state {
		// key down
		if let Some(code) = event.virtual_keycode {
			println!("down: {:?}", code);
			match code {
				winit::event::VirtualKeyCode::Escape => {
					std::process::exit(0);
				},
				winit::event::VirtualKeyCode::LShift => {
					player.position_y -= 1 << 7;
				},
				winit::event::VirtualKeyCode::Space => {
					player.position_y += 1 << 7;
				},
				winit::event::VirtualKeyCode::A => {
					player.position_x -= 1 << 7;
				},
				winit::event::VirtualKeyCode::D => {
					player.position_x += 1 << 7;
				},
				winit::event::VirtualKeyCode::S => {
					player.position_z -= 1 << 7;
				},
				winit::event::VirtualKeyCode::W => {
					player.position_z += 1 << 7;
				},
				winit::event::VirtualKeyCode::Left => {
					player.angle_h -= 0.1;
				},
				winit::event::VirtualKeyCode::Right => {
					player.angle_h += 0.1;
				},
				winit::event::VirtualKeyCode::Up => {
					player.angle_v -= 0.1;
				},
				winit::event::VirtualKeyCode::Down => {
					player.angle_v += 0.1;
				},
				_ => {},
			}
			println!(
				"player position: {} {} {}",
				player.position_x as f32 / 256.0,
				player.position_y as f32 / 256.0,
				player.position_z as f32 / 256.0,
			);
		}
	}
	else {
		// key up
		if let Some(code) = event.virtual_keycode {
			match code {
				_ => {},
			}
		}
	}
}
