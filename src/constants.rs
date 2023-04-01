#![allow(dead_code)]

pub const CHUNK_WIDTH_L2: u8 = 4;
pub const CHUNK_HEIGHT_FACTOR_L2: u8 = 2;
pub const CHUNK_HEIGHT_L2: u8 = CHUNK_WIDTH_L2 + CHUNK_HEIGHT_FACTOR_L2;

pub const CHUNK_WIDTH: usize = 1 << CHUNK_WIDTH_L2;
pub const CHUNK_HEIGHT_FACTOR: usize = 1 << CHUNK_HEIGHT_FACTOR_L2;
pub const CHUNK_HEIGHT: usize = 1 << CHUNK_HEIGHT_L2;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockType {
	Air = 0,
	Stone = 1,
	Grass = 2,
	Dirt = 3,
	Cobble = 4,
	Planks = 5,
	Bedrock = 6,
	Log = 7,
	Leaves = 8,
	Bricks = 9,
	Wool = 10,
	Sand = 11,
	Gravel = 12,
	Glass = 13,
	Bookshelf = 14,
	Obsidian = 15,
	StoneBricks = 16,
	Sandstone = 17,
}

// TODO get this dynamically
pub const BLOCK_TYPE_MAX: BlockType = BlockType::Sandstone;

impl From<BlockType> for u8 {
	fn from(block_type: BlockType) -> Self {
		block_type as u8
	}
}
impl Into<BlockType> for u8 {
	fn into(self) -> BlockType {
		assert!(self <= BLOCK_TYPE_MAX as u8, "invalid block type");
		unsafe { std::mem::transmute(self) }
	}
}

pub const WORLD_FLATMAP_TEMPLATE: [BlockType; 7] = [
	BlockType::Bedrock,
	BlockType::Stone,
	BlockType::Stone,
	BlockType::Dirt,
	BlockType::Dirt,
	BlockType::Dirt,
	BlockType::Grass,
];
