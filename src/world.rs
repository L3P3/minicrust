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
		(
			(x as usize) << CHUNK_WIDTH_L2 | z as usize
		) << CHUNK_HEIGHT_L2 | y as usize
	}

	#[inline]
	pub unsafe fn block_get_unchecked_index (&self, index: usize) -> BlockType {
		*self.blocks.get_unchecked(index)
	}

	#[inline]
	pub unsafe fn block_get_unchecked(&self, x: u16, y: u8, z: u16) -> BlockType {
		self.block_get_unchecked_index(Self::block_index_get(x, y, z))
	}

	#[allow(dead_code)]//todo
	pub fn block_get(&self, x: u16, y: u8, z: u16) -> BlockType {
		if y < CHUNK_HEIGHT as u8 &&
			x < CHUNK_WIDTH as u16 &&
			z < CHUNK_WIDTH as u16 {
			unsafe {
				self.block_get_unchecked(x, y, z)
			}
		}
		else {
			BlockType::Air
		}
	}

	pub fn chunk_generate(&mut self) {
		for (counter, strip) in self.blocks.chunks_exact_mut(CHUNK_HEIGHT).enumerate() {
			strip[..WORLD_FLATMAP_TEMPLATE.len()]
				.copy_from_slice(&WORLD_FLATMAP_TEMPLATE);

			if counter & 0x10 == 0 && counter & 0x1 == 0 {
				strip[10] = BlockType::Wool;
			}
		}
	}
}
