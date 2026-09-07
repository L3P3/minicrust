use winit::keyboard::KeyCode;

use crate::player;

pub fn key_input(
	event: &winit::event::KeyEvent,
	player: &mut player::Player,
) {
	let winit::keyboard::PhysicalKey::Code(code) = event.physical_key else {
		return;
	};

	if event.state == winit::event::ElementState::Pressed {
		println!("down: {:?}", code);
		match code {
			KeyCode::Escape => {
				std::process::exit(0);
			},
			KeyCode::ShiftLeft => {
				player.position_y -= 1 << 7;
			},
			KeyCode::Space => {
				player.position_y += 1 << 7;
			},
			KeyCode::KeyA => {
				player.position_x -= 1 << 7;
			},
			KeyCode::KeyD => {
				player.position_x += 1 << 7;
			},
			KeyCode::KeyS => {
				player.position_z -= 1 << 7;
			},
			KeyCode::KeyW => {
				player.position_z += 1 << 7;
			},
			KeyCode::ArrowLeft => {
				player.angle_h -= 0.1;
			},
			KeyCode::ArrowRight => {
				player.angle_h += 0.1;
			},
			KeyCode::ArrowUp => {
				player.angle_v -= 0.1;
			},
			KeyCode::ArrowDown => {
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
