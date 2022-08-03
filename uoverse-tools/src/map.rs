use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    convert::{TryFrom, TryInto},
    io::{Cursor, Read, Write},
    mem::size_of,
};

use crate::archive::uo_package::{self, FileType, UOPackage, UOPackageFile};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("map size ({width},{height}) is invalid")]
    InvalidSize { width: u32, height: u32 },

    #[error("map position ({x},{y}) is invalid")]
    InvalidPos { x: u32, y: u32 },

    #[error("map file in package is invalid because {0}")]
    InvalidFile(#[from] uo_package::Error),

    #[error("no map files in package")]
    NoFiles,

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
    const SIZE: usize = size_of::<u16>() + size_of::<u8>();

    fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self {
            kind: reader.read_u16::<LittleEndian>()?,
            height: reader.read_u8()?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16::<LittleEndian>(self.kind)?;
        writer.write_u8(self.height)?;

        Ok(())
    }
}

pub struct Block<const BLOCK_SIZE: u32>
where
    [(); BLOCK_SIZE as usize]:,
{
    id: u32, // ClassicUO calls this "Header" but doesn't seem to use it
    tiles: [[Tile; BLOCK_SIZE as usize]; BLOCK_SIZE as usize], // 2D array, y-major
}

impl<const BLOCK_SIZE: u32> Block<BLOCK_SIZE>
where
    [(); BLOCK_SIZE as usize]:,
{
    const SIZE: usize = size_of::<u32>() + (Tile::SIZE * BLOCK_SIZE as usize * BLOCK_SIZE as usize);

    fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut block = Self {
            id: reader.read_u32::<LittleEndian>()?,
            tiles: [[Default::default(); BLOCK_SIZE as usize]; BLOCK_SIZE as usize],
        };

        for y in 0..(BLOCK_SIZE as usize) {
            for x in 0..(BLOCK_SIZE as usize) {
                block.tiles[y][x] = Tile::from_reader(reader)?;
            }
        }

        Ok(block)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.id)?;

        for tiles_y in self.tiles {
            for tile in tiles_y {
                tile.write(writer)?;
            }
        }

        Ok(())
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
    const BLOCKS_PER_FILE: usize = 0x1000;

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
                id,
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

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for block in &self.blocks {
            block.write(writer)?;
        }

        Ok(())
    }

    pub fn into_files(self, prefix: &str) -> Result<Vec<UOPackageFile>> {
        let num_files = (self.blocks.len() + Self::BLOCKS_PER_FILE - 1) / Self::BLOCKS_PER_FILE;

        fn write_blocks_file<const BLOCK_SIZE: u32>(
            blocks: &[Block<BLOCK_SIZE>],
            file_name: &str,
        ) -> Result<UOPackageFile>
        where
            [(); BLOCK_SIZE as usize]:,
        {
            let size =
                Block::<BLOCK_SIZE>::SIZE * blocks.len().min(Map::<BLOCK_SIZE>::BLOCKS_PER_FILE);
            let mut contents = vec![0u8; size];
            let mut buf = contents.as_mut_slice();

            for block in blocks {
                block.write(&mut buf)?;
            }

            Ok(UOPackageFile {
                hash: uo_package::uop_hash(file_name)?,
                file_type: FileType::MapTiles,
                timestamp: None,
                contents,
            })
        }

        // Break the map up into separate files
        let mut files = Vec::<UOPackageFile>::with_capacity(num_files);
        let mut file_num = 0u32;

        let mut block_chunks = self.blocks.chunks_exact(Self::BLOCKS_PER_FILE);
        for chunk in block_chunks.by_ref() {
            let file_name = Self::file_path(prefix, file_num);
            files.push(write_blocks_file(chunk, file_name.as_str())?);
            file_num += 1;
        }

        let remainder = block_chunks.remainder();
        if !remainder.is_empty() {
            let file_name = Self::file_path(prefix, file_num);
            files.push(write_blocks_file(remainder, file_name.as_str())?);
        }

        Ok(files)
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

    fn file_path(prefix: &str, file_num: u32) -> String {
        format!("{}/{:08}.dat", prefix, file_num)
    }
}

pub struct Metadata {
    pub width: u32,
    pub height: u32,
    pub prefix: String,
}

impl<const BLOCK_SIZE: u32> TryFrom<(Metadata, UOPackage)> for Map<BLOCK_SIZE>
where
    [(); BLOCK_SIZE as usize]:,
{
    type Error = Error;

    fn try_from((metadata, package): (Metadata, UOPackage)) -> Result<Self> {
        let mut reader: PackageReader<'_> = (metadata.prefix.as_str(), &package).try_into()?;
        Ok(Self::from_reader(
            &mut reader,
            metadata.width,
            metadata.height,
        )?)
    }
}

// UO maps use 8x8 blocks
pub type UOMap = Map<8>;

pub struct PackageReader<'a> {
    package: &'a UOPackage,
    prefix: String,
    file_num: u32,
    inner: Cursor<&'a [u8]>,
}

impl<'a> PackageReader<'a> {
    fn file_path(prefix: &str, file_num: u32) -> String {
        Map::<0>::file_path(prefix, file_num)
    }

    fn get_next_file(&self) -> Result<Option<&'a UOPackageFile>> {
        Ok(self
            .package
            .get_file(Self::file_path(self.prefix.as_str(), self.file_num + 1).as_str())?)
    }
}

impl<'a, 'b> TryFrom<(&'b str, &'a UOPackage)> for PackageReader<'a> {
    type Error = Error;

    fn try_from((prefix, package): (&'b str, &'a UOPackage)) -> Result<Self> {
        let file_num = 0;

        match package.get_file(Self::file_path(prefix, file_num).as_str())? {
            Some(file) => Ok(Self {
                package,
                prefix: prefix.to_string(),
                file_num,
                inner: Cursor::new(&file.contents),
            }),
            None => Err(Error::NoFiles),
        }
    }
}

impl<'a> Read for PackageReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut amount = self.inner.read(buf)?;

        if amount == 0 {
            if let Some(file) = self
                .get_next_file()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            {
                self.inner = Cursor::new(&file.contents);
                self.file_num += 1;

                amount = self.inner.read(buf)?;
            }
        }

        Ok(amount)
    }
}
