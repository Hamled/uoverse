use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::{
    convert::TryInto,
    fmt,
    io::{Read, Seek, SeekFrom},
};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("package header magic ({0:?}) is invalid")]
    InvalidMagic([u8; 4]),

    #[error("package version {0} is not supported")]
    UnsupportedVersion(u32),

    #[error("hash input encoding is not supported")]
    UnsupportedEncoding,

    #[error("hash input is too large ({0} bytes)")]
    InputTooLarge(usize),

    #[error("hash input is too small")]
    InputTooSmall,

    #[error("i/o failure {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

const HEADER_MAGIC: [u8; 4] = [0x4D, 0x59, 0x50, 0x00];

#[derive(Debug)]
pub struct PackageHdr {
    version: u32,
    _format: u32,
    first_block: u64,
    _block_size: u32,
    _files_count: u32,
}

impl PackageHdr {
    fn new<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Verify
        let mut header = [0u8; 4];
        reader.read_exact(header.as_mut_slice())?;

        if header != HEADER_MAGIC {
            return Err(Error::InvalidMagic(header));
        }

        Ok(PackageHdr {
            version: reader.read_u32::<LittleEndian>()?,
            _format: reader.read_u32::<LittleEndian>()?,
            first_block: reader.read_u64::<LittleEndian>()?,
            _block_size: reader.read_u32::<LittleEndian>()?,
            _files_count: reader.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug)]
struct BlockHdr {
    _files_count: u32,
    next_block: u64,
    headers: Vec<FileHdr>,
}

impl BlockHdr {
    fn new<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let files_count = reader.read_u32::<LittleEndian>()?;
        let next_block = reader.read_u64::<LittleEndian>()?;

        let mut headers = Vec::<FileHdr>::with_capacity(files_count as usize);
        for _ in 0..files_count {
            headers.push(FileHdr::new(reader)?);
        }

        Ok(BlockHdr {
            _files_count: files_count,
            next_block,
            headers,
        })
    }
}

#[derive(Debug)]
struct FileHdr {
    position: u64,
    _header_size: u32,
    compressed_size: u32,
    raw_size: u32,
    hash: u64,
    _header_crc: u32,
    entry_type: u16,
}

impl FileHdr {
    fn new<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(FileHdr {
            position: reader.read_u64::<LittleEndian>()?,
            _header_size: reader.read_u32::<LittleEndian>()?,
            compressed_size: reader.read_u32::<LittleEndian>()?,
            raw_size: reader.read_u32::<LittleEndian>()?,
            hash: reader.read_u64::<LittleEndian>()?,
            _header_crc: reader.read_u32::<LittleEndian>()?,
            entry_type: reader.read_u16::<LittleEndian>()?,
        })
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u16)]
pub enum FileType {
    MapTiles = 4,
    Unknown = 0xFFFF,
}

impl From<u16> for FileType {
    fn from(val: u16) -> Self {
        match val {
            4 => Self::MapTiles,
            _ => Self::Unknown,
        }
    }
}

pub struct UOPackageFile {
    pub hash: u64,
    pub file_type: FileType,
    header_remaining: u16,
    timestamp: u64,
    pub contents: Vec<u8>,
}

impl UOPackageFile {
    fn read_version4<R: Read>(reader: &mut R, header: &FileHdr) -> Result<Self> {
        let mut file = UOPackageFile {
            hash: header.hash,
            file_type: reader.read_u16::<LittleEndian>()?.into(),
            header_remaining: reader.read_u16::<LittleEndian>()?,
            timestamp: reader.read_u64::<LittleEndian>()?,
            contents: Vec::with_capacity(header.raw_size as usize),
        };

        assert!(file.header_remaining as usize == std::mem::size_of::<u64>());

        Self::read_contents(reader, header, &mut file.contents)?;
        Ok(file)
    }

    fn read_version5<R: Read>(reader: &mut R, header: &FileHdr) -> Result<Self> {
        // Header is unknown, read it and ignore
        // TODO: Verify header CRC
        let reader = {
            let mut buf = Vec::<u8>::with_capacity(header.header_size as usize);
            let mut reader = reader.take(header.header_size.into());
            reader.read_to_end(&mut buf)?;
            reader.into_inner()
        };

        let mut file = UOPackageFile {
            hash: header.hash,
            file_type: FileType::Unknown,
            header_remaining: 0,
            timestamp: 0,
            contents: Vec::with_capacity(header.raw_size as usize),
        };

        Self::read_contents(reader, header, &mut file.contents)?;
        Ok(file)
    }

    fn read_contents<R: Read>(
        reader: &mut R,
        header: &FileHdr,
        contents: &mut Vec<u8>,
    ) -> Result<()> {
        match header.entry_type {
            0 => {
                let mut reader = reader.take(header.raw_size.into());
                let amount = reader.read_to_end(contents)?;
                assert!(amount == header.raw_size as usize);
            }
            1 => {
                let reader = reader.take(header.compressed_size.into());
                let mut decoder = ZlibDecoder::new(reader);
                decoder.read_to_end(contents)?;
                assert!(decoder.total_in() == header.compressed_size.into());
                assert!(decoder.total_out() == header.raw_size.into());
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    fn new<R: Read>(reader: &mut R, header: &FileHdr, version: u32) -> Result<Self> {
        match version {
            4 => Self::read_version4(reader, header),
            5 => Self::read_version5(reader, header),
            _ => Err(Error::UnsupportedVersion(version)),
        }
    }
}

impl fmt::Debug for UOPackageFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "File(hash: {:016X}, type: {:?}, length: {})",
            self.hash,
            self.file_type,
            self.contents.len()
        ))
    }
}

#[derive(Debug)]
pub struct UOPackage {
    header: PackageHdr,
    pub files: Vec<UOPackageFile>,
}

impl UOPackage {
    pub fn new<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = PackageHdr::new(reader)?;

        match header.version {
            4 | 5 => {}
            _ => return Err(Error::UnsupportedVersion(header.version)),
        }

        let mut package = UOPackage {
            header,
            files: vec![],
        };

        package.read_files(reader)?;
        Ok(package)
    }

    pub fn get_file<'a>(&'a self, path: &str) -> Result<Option<&'a UOPackageFile>> {
        let hash = uop_hash(path)?;
        Ok(self.files.iter().find(|f| f.hash == hash))
    }

    fn read_files<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()> {
        // Read all of the block headers
        let mut block_pos = self.header.first_block;
        while block_pos != 0 {
            reader.seek(SeekFrom::Start(block_pos))?;
            let block = BlockHdr::new(reader)?;
            for header in block.headers {
                if header.position == 0 {
                    continue;
                }

                reader.seek(SeekFrom::Start(header.position))?;
                self.files
                    .push(UOPackageFile::new(reader, &header, self.header.version)?);
            }

            block_pos = block.next_block;
        }

        Ok(())
    }
}

// UOP file name hash algorithm adapted from
// https://github.com/ClassicUO/ClassicUO/blob/69857dc07b5d84ecf0e404df3fe3c8514df3a4c7/src/IO/UOFileUop.cs#L198
// which turns out to just be lookup3 from Bob Jenkins:
// http://www.burtleburtle.net/bob/hash/doobs.html
fn uop_hash(input: &str) -> Result<u64> {
    if input.is_empty() {
        return Err(Error::InputTooSmall);
    }

    if !input.is_ascii() {
        return Err(Error::UnsupportedEncoding);
    }

    Ok(hashers::jenkins::lookup3(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_map_paths() {
        let input = "build/map4legacymul/00000000.dat";
        let output = uop_hash(input).unwrap();

        assert_eq!(output, 0xDEA39C8655BA717C);
    }
}
