#![allow(dead_code)]

pub const CHUNK_WIDTH_L2: u8 = 4;
pub const CHUNK_HEIGHT_FACTOR_L2: u8 = 2;
pub const CHUNK_HEIGHT_L2: u8 = CHUNK_WIDTH_L2 + CHUNK_HEIGHT_FACTOR_L2;

pub const CHUNK_WIDTH: usize = 1 << CHUNK_WIDTH_L2;
pub const CHUNK_WIDTH_MASK: usize = CHUNK_WIDTH - 1;
pub const CHUNK_HEIGHT_FACTOR: usize = 1 << CHUNK_HEIGHT_FACTOR_L2;
pub const CHUNK_HEIGHT: usize = 1 << CHUNK_HEIGHT_L2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Color {
	b: u8,
	g: u8,
	r: u8,
	a: u8,
}

impl Color {
	#[inline(always)]
	pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b, a: 0x00 }
	}

	/// scale color channels by [0..1] in q8
	pub fn scale(&self, factor: u16) -> Self {
		debug_assert!(factor <= 0x100, "invalid color scale factor");
		Self {
			r: (self.r as u16 * factor >> 8) as u8,
			g: (self.g as u16 * factor >> 8) as u8,
			b: (self.b as u16 * factor >> 8) as u8,
			a: self.a,
		}
	}
}

#[derive(Clone, Copy)]
pub enum BlockFace {
	East,
	West,
	Top,
	Bottom,
	North,
	South,
}

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
	LapisBlock = 18,
	IronBlock = 19,
	GoldBlock = 20,
	DiamondBlock = 21,
	EmeraldBlock = 22,
	RedstoneBlock = 23,
	QuartzBlock = 24,
	Count,
}

impl From<BlockType> for u8 {
	fn from(block_type: BlockType) -> Self {
		block_type as u8
	}
}
impl From<u8> for BlockType {
	fn from(block_id: u8) -> BlockType {
		assert!(block_id < BlockType::Count as u8, "invalid block type");
		unsafe { std::mem::transmute(block_id) }
	}
}

// Entries must remain in the same order as the `#[repr(u8)]` BlockType variants.
const BLOCK_COLORS: [Color; BlockType::Count as usize] = [
	Color::rgb(0x00, 0x00, 0x00),
	Color::rgb(0x81, 0x81, 0x81),
	Color::rgb(0x72, 0xb1, 0x41),
	Color::rgb(0x81, 0x5d, 0x42),
	Color::rgb(0x7b, 0x7b, 0x7b),
	Color::rgb(0xa2, 0x83, 0x51),
	Color::rgb(0x6b, 0x6b, 0x6b),
	Color::rgb(0x6f, 0x58, 0x36),
	Color::rgb(0x6e, 0xac, 0x3f),
	Color::rgb(0x95, 0x6b, 0x61),
	Color::rgb(0xe8, 0xeb, 0xec),
	Color::rgb(0xdc, 0xd4, 0xa2),
	Color::rgb(0x7e, 0x7c, 0x7b),
	Color::rgb(0xe0, 0xf6, 0xfa),
	Color::rgb(0x78, 0x59, 0x3f),
	Color::rgb(0x14, 0x12, 0x1e),
	Color::rgb(0x7b, 0x7b, 0x7b),
	Color::rgb(0xd3, 0xcc, 0x93),
	Color::rgb(0x27, 0x43, 0x8b),
	Color::rgb(0xe7, 0xe7, 0xe7),
	Color::rgb(0xfd, 0xf3, 0x54),
	Color::rgb(0x7e, 0xe1, 0xdd),
	Color::rgb(0x55, 0xdb, 0x78),
	Color::rgb(0xac, 0x1d, 0x0a),
	Color::rgb(0xed, 0xea, 0xe3),
];

#[inline(always)]
pub fn block_color_get(block_type: BlockType) -> Color {
	BLOCK_COLORS[usize::from(u8::from(block_type))]
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
