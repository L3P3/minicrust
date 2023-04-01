use crate::constants::*;

pub struct World {
	blocks: Vec<BlockType>,
}

impl World {
	pub fn new() -> Self {
		let mut instance = Self {
			blocks: vec![BlockType::Air; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT],
		};

		instance.chunk_generate();

		instance
	}

	fn block_index_get(x: usize, y: usize, z: usize) -> usize {
		(
			x << CHUNK_WIDTH_L2 | z
		) << CHUNK_HEIGHT_L2 | y
	}

	pub fn block_get(&self, x: usize, y: usize, z: usize) -> BlockType {
		self.blocks[Self::block_index_get(x, y, z)]
	}

	pub fn block_get_checked(&self, x: i32, y: i32, z: i32) -> BlockType {
		if x >= 0 && x < CHUNK_WIDTH as i32 &&
			y >= 0 && y < CHUNK_HEIGHT as i32 &&
			z >= 0 && z < CHUNK_WIDTH as i32 {
			self.block_get(x as usize, y as usize, z as usize)
		}
		else {
			BlockType::Air
		}
	}

	pub fn chunk_generate(&mut self) {
		for strip in self.blocks.chunks_exact_mut(CHUNK_HEIGHT) {
			strip[..WORLD_FLATMAP_TEMPLATE.len()]
				.copy_from_slice(&WORLD_FLATMAP_TEMPLATE);
		}
	}
}
