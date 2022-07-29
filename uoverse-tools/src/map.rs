use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("map size ({width},{height}) is invalid")]
    InvalidSize { width: u32, height: u32 },

    #[error("map position ({x},{y}) is invalid")]
    InvalidPos { x: u32, y: u32 },

    #[error("i/o failure {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Default)]
pub struct Tile {
    pub kind: u16,
    pub height: u8,
}

impl Tile {
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self {
            kind: reader.read_u16::<LittleEndian>()?,
            height: reader.read_u8()?,
        })
    }
}

pub struct Block<const BLOCK_SIZE: u32>
where
    [(); BLOCK_SIZE as usize]:,
{
    _id: u32, // ClassicUO calls this "Header" but doesn't seem to use it
    tiles: [[Tile; BLOCK_SIZE as usize]; BLOCK_SIZE as usize], // 2D array, y-major
}

impl<const BLOCK_SIZE: u32> Block<BLOCK_SIZE>
where
    [(); BLOCK_SIZE as usize]:,
{
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut block = Self {
            _id: reader.read_u32::<LittleEndian>()?,
            tiles: [[Default::default(); BLOCK_SIZE as usize]; BLOCK_SIZE as usize],
        };

        for y in 0..(BLOCK_SIZE as usize) {
            for x in 0..(BLOCK_SIZE as usize) {
                block.tiles[y][x] = Tile::from_reader(reader)?;
            }
        }

        Ok(block)
    }
}

pub struct Map<const BLOCK_SIZE: u32>
where
    [(); BLOCK_SIZE as usize]:,
{
    width: u32,
    height: u32,
    blocks: Vec<Block<BLOCK_SIZE>>, // 2D array, width-major
}

impl<const BLOCK_SIZE: u32> Map<BLOCK_SIZE>
where
    [(); BLOCK_SIZE as usize]:,
{
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Self::validate_dimensions(width, height)?;

        let blocks_num = (width * height) / (BLOCK_SIZE * BLOCK_SIZE);
        let mut map = Self {
            width,
            height,
            blocks: Vec::with_capacity(blocks_num as usize),
        };

        for id in 0..blocks_num {
            map.blocks.push(Block {
                _id: id,
                tiles: [[Default::default(); BLOCK_SIZE as usize]; BLOCK_SIZE as usize],
            })
        }

        Ok(map)
    }

    pub fn from_reader<R: Read>(reader: &mut R, width: u32, height: u32) -> Result<Self> {
        Self::validate_dimensions(width, height)?;

        let blocks_num = (width * height) / (BLOCK_SIZE * BLOCK_SIZE);
        let mut map = Self {
            width,
            height,
            blocks: Vec::with_capacity(blocks_num as usize),
        };

        for _ in 0..blocks_num {
            map.blocks.push(Block::from_reader(reader)?);
        }

        Ok(map)
    }

    pub fn set(&mut self, x: u32, y: u32, tile: Tile) -> Result<()> {
        self.validate_position(x, y)?;

        let block_x = x / BLOCK_SIZE;
        let block_y = y / BLOCK_SIZE;
        let block = &mut self.blocks[(block_x * (self.height / BLOCK_SIZE) + block_y) as usize];

        let tile_x = x % BLOCK_SIZE;
        let tile_y = y % BLOCK_SIZE;
        block.tiles[tile_y as usize][tile_x as usize] = tile;
        Ok(())
    }

    pub fn get(&self, x: u32, y: u32) -> Result<&Tile> {
        self.validate_position(x, y)?;

        let block_x = x / BLOCK_SIZE;
        let block_y = y / BLOCK_SIZE;
        let block = &self.blocks[(block_x * (self.height / BLOCK_SIZE) + block_y) as usize];

        let tile_x = x % BLOCK_SIZE;
        let tile_y = y % BLOCK_SIZE;
        Ok(&block.tiles[tile_y as usize][tile_x as usize])
    }

    fn validate_dimensions(width: u32, height: u32) -> Result<()> {
        // Map must be composed of square blocks, does not have to be square
        if width == 0 || height == 0 || width % BLOCK_SIZE != 0 || height % BLOCK_SIZE != 0 {
            Err(Error::InvalidSize { width, height })
        } else {
            Ok(())
        }
    }

    fn validate_position(&self, x: u32, y: u32) -> Result<()> {
        if x >= self.width || y >= self.height {
            Err(Error::InvalidPos { x, y })
        } else {
            Ok(())
        }
    }
}

// UO maps use 8x8 blocks
pub type UOMap = Map<8>;
