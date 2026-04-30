use crate::constants::*;

pub struct World {
	blocks: Vec<BlockType>,
	pub spawn_y: i16,
	pub spawn_x: i32,
	pub spawn_z: i32,
}

impl World {
	pub fn new() -> Self {
		let mut instance = Self {
			blocks: vec![BlockType::Air; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT],
			spawn_y: 13 << 8,
			spawn_x: 0 << 8,
			spawn_z: 0 << 8,
		};

		instance.chunk_generate();

		instance
	}

	#[inline]
	fn block_index_get(x: u16, y: u8, z: u16) -> usize {
		assert!(y < CHUNK_HEIGHT as u8);
		// TODO still assumes a single chunk width
		// ((y as usize >> CHUNK_WIDTH_L2) & (CHUNK_HEIGHT / CHUNK_WIDTH - 1)) << (CHUNK_WIDTH_L2 * 3) |
		// ((x as usize >> CHUNK_WIDTH_L2) & CHUNK_WIDTH_MASK) << (CHUNK_WIDTH_L2 * 4) |
		// ((z as usize >> CHUNK_WIDTH_L2) & CHUNK_WIDTH_MASK) << (CHUNK_WIDTH_L2 * 3) |
		// ((y as usize) & CHUNK_WIDTH_MASK) << (CHUNK_WIDTH_L2 * 2) |
		// ((x as usize) & CHUNK_WIDTH_MASK) << CHUNK_WIDTH_L2 |
		// (z as usize) & CHUNK_WIDTH_MASK
		(y as usize) << (CHUNK_WIDTH_L2 * 2) |
		((x as usize) & CHUNK_WIDTH_MASK) << CHUNK_WIDTH_L2 |
		(z as usize) & CHUNK_WIDTH_MASK
	}

	#[inline]
	pub fn block_get(&self, x: u16, y: u8, z: u16) -> BlockType {
		// save because block_index_get bounds checks
		unsafe {
			*self.blocks.get_unchecked(Self::block_index_get(x, y, z))
		}
	}

	pub fn chunk_generate(&mut self) {
		for x in 0..CHUNK_WIDTH as u16 {
			for z in 0..CHUNK_WIDTH as u16 {
				for (y, block) in WORLD_FLATMAP_TEMPLATE.iter().copied().enumerate() {
					let index = Self::block_index_get(x, y as u8, z);
					self.blocks[index] = block;
				}

				let counter = (x as usize) << CHUNK_WIDTH_L2 | z as usize;
				if counter & 0x10 == 0 && counter & 0x1 == 0 {
					let index = Self::block_index_get(x, 10, z);
					self.blocks[index] = BlockType::Wool;
				}
			}
		}
	}
}
