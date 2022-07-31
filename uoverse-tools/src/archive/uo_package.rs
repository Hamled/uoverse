use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::{
    convert::TryInto,
    fmt,
    io::{Read, Seek, SeekFrom, Write},
    mem::size_of,
};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("package header magic ({0:?}) is invalid")]
    InvalidMagic([u8; 4]),

    #[error("package data is invalid because {0}")]
    InvalidData(String),

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
const FORMAT_MAGIC: u32 = 0xFD23EC43;

// When writing, align block headers & file entries on 4K page boundaries
const ALIGNMENT: u64 = 0x1000;

#[derive(Debug)]
pub struct PackageHdr {
    version: u32,
    format: u32,
    first_block: u64,
    block_size: u32,
    files_count: u32,
}

impl PackageHdr {
    fn new<R: Read>(reader: &mut R) -> Result<Self> {
        // Verify
        let mut header = [0u8; 4];
        reader.read_exact(header.as_mut_slice())?;

        if header != HEADER_MAGIC {
            return Err(Error::InvalidMagic(header));
        }

        Ok(PackageHdr {
            version: reader.read_u32::<LittleEndian>()?,
            format: reader.read_u32::<LittleEndian>()?,
            first_block: reader.read_u64::<LittleEndian>()?,
            block_size: reader.read_u32::<LittleEndian>()?,
            files_count: reader.read_u32::<LittleEndian>()?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&HEADER_MAGIC)?;

        writer.write_u32::<LittleEndian>(self.version)?;
        writer.write_u32::<LittleEndian>(self.format)?;
        writer.write_u64::<LittleEndian>(self.first_block)?;
        writer.write_u32::<LittleEndian>(self.block_size)?;
        writer.write_u32::<LittleEndian>(self.files_count)?;

        Ok(())
    }
}

impl Default for PackageHdr {
    fn default() -> Self {
        Self {
            version: 5,
            format: FORMAT_MAGIC,
            first_block: 0x200,
            block_size: 100, // Can fit at most 119 file headers per 4K page
            files_count: 0,
        }
    }
}

#[derive(Debug)]
struct BlockHdr {
    files_count: u32,
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
            files_count,
            next_block,
            headers,
        })
    }

    fn size(num_headers: u32) -> usize {
        size_of::<u32>() + size_of::<u64>() + (FileHdr::SIZE * num_headers as usize)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.files_count)?;
        writer.write_u64::<LittleEndian>(self.next_block)?;

        for header in &self.headers {
            header.write(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct FileHdr {
    position: u64,
    header_size: u32,
    compressed_size: u32,
    raw_size: u32,
    hash: u64,
    _header_crc: u32,
    entry_type: u16,
}

impl FileHdr {
    const SIZE: usize = (size_of::<u64>() * 2) + (size_of::<u32>() * 4) + size_of::<u16>();

    fn new<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(FileHdr {
            position: reader.read_u64::<LittleEndian>()?,
            header_size: reader.read_u32::<LittleEndian>()?,
            compressed_size: reader.read_u32::<LittleEndian>()?,
            raw_size: reader.read_u32::<LittleEndian>()?,
            hash: reader.read_u64::<LittleEndian>()?,
            _header_crc: reader.read_u32::<LittleEndian>()?,
            entry_type: reader.read_u16::<LittleEndian>()?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LittleEndian>(self.position)?;
        writer.write_u32::<LittleEndian>(self.header_size)?;
        writer.write_u32::<LittleEndian>(self.compressed_size)?;
        writer.write_u32::<LittleEndian>(self.raw_size)?;
        writer.write_u64::<LittleEndian>(self.hash)?;
        writer.write_u32::<LittleEndian>(self._header_crc)?;
        writer.write_u16::<LittleEndian>(self.entry_type)?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u16)]
pub enum FileType {
    Compressed = 3,
    MapTiles = 4,
    Unknown = 0xFFFF,
}

impl FileType {
    fn is_compressed(&self) -> bool {
        match self {
            Self::Compressed => true,
            _ => false,
        }
    }
}

impl From<u16> for FileType {
    fn from(val: u16) -> Self {
        match val {
            3 => Self::Compressed,
            4 => Self::MapTiles,
            _ => Self::Unknown,
        }
    }
}

pub struct UOPackageFile {
    pub hash: u64,
    pub file_type: FileType,
    pub timestamp: Option<u64>,
    pub contents: Vec<u8>,
}

impl UOPackageFile {
    const HEADER_SIZE_V4: usize = 12;
    fn read_version4<R: Read + Seek>(reader: &mut R, header: &FileHdr) -> Result<Self> {
        let file_type = reader.read_u16::<LittleEndian>()?.into();
        let remaining = reader.read_u16::<LittleEndian>()?;
        let timestamp = Some(reader.read_u64::<LittleEndian>()?);

        // Skip rest of header
        let remaining = (remaining as usize).checked_sub(size_of::<u64>());
        if remaining.is_none() {
            return Err(Error::InvalidData(format!(
                "metadata for file {:016X} is invalid",
                header.hash
            )));
        }
        reader.seek(SeekFrom::Current(remaining.unwrap() as i64))?;

        // TODO: Verify header CRC
        let mut file = UOPackageFile {
            hash: header.hash,
            file_type,
            timestamp,
            contents: Vec::with_capacity(header.raw_size as usize),
        };

        Self::read_contents(reader, header, &mut file.contents)?;
        Ok(file)
    }

    const HEADER_SIZE_V5: usize = 137; // What the UOLive files have
    fn read_version5<R: Read + Seek>(reader: &mut R, header: &FileHdr) -> Result<Self> {
        let file_type = reader.read_u16::<LittleEndian>()?.into();
        let remaining = reader.read_u16::<LittleEndian>()?;

        // Rest of header is unknown, skip it
        reader.seek(SeekFrom::Current(remaining as i64))?;

        // TODO: Verify header CRC
        let mut file = UOPackageFile {
            hash: header.hash,
            file_type,
            timestamp: None,
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

    fn new<R: Read + Seek>(reader: &mut R, header: &FileHdr, version: u32) -> Result<Self> {
        match version {
            4 => Self::read_version4(reader, header),
            5 => Self::read_version5(reader, header),
            _ => Err(Error::UnsupportedVersion(version)),
        }
    }

    fn write_header_v4<W: Write + Seek>(&self, writer: &mut W) -> Result<usize> {
        let preamble_size = size_of::<u16>() * 2; // file_type and remaining
        let remaining =
            Self::HEADER_SIZE_V4
                .checked_sub(preamble_size)
                .ok_or(Error::InvalidData(format!(
                    "metadata for file {:016X} is larger than expected",
                    self.hash
                )))?;

        writer.write_u16::<LittleEndian>(self.file_type as u16)?;
        writer.write_u16::<LittleEndian>(remaining as u16)?;

        writer.write_u64::<LittleEndian>(self.timestamp.unwrap_or(0))?;
        Ok(remaining
            .checked_sub(size_of::<u64>()) // Already wrote timestamp
            .ok_or(Error::InvalidData(format!(
                "metadata for file {:016X} is larger than expected",
                self.hash
            )))?)
    }

    fn write_header_v5<W: Write + Seek>(&self, writer: &mut W) -> Result<usize> {
        let preamble_size = size_of::<u16>() * 2; // file_type and remaining
        let remaining =
            Self::HEADER_SIZE_V5
                .checked_sub(preamble_size)
                .ok_or(Error::InvalidData(format!(
                    "metadata for file {:016X} is larger than expected",
                    self.hash
                )))?;

        writer.write_u16::<LittleEndian>(self.file_type as u16)?;
        writer.write_u16::<LittleEndian>(remaining as u16)?;

        Ok(remaining)
    }

    fn write_header<W: Write + Seek>(&self, writer: &mut W, version: u32) -> Result<usize> {
        match version {
            4 => self.write_header_v4(writer),
            5 => self.write_header_v5(writer),
            _ => Err(Error::UnsupportedVersion(version)),
        }
    }

    fn write<W: Write + Seek>(&self, writer: &mut W, version: u32) -> Result<(u32, u32)> {
        // Write the file metadata first
        let remaining = self.write_header(writer, version)?;

        // Write the file content
        writer.seek(SeekFrom::Current(remaining as i64))?;

        match self.file_type.is_compressed() {
            true => {
                let mut encoder = ZlibEncoder::new(writer, Compression::best());
                encoder.write_all(self.contents.as_slice())?;
                encoder.try_finish()?;

                let compressed_size = encoder.total_out() as u32;
                let raw_size = encoder.total_in() as u32;
                Ok((compressed_size, raw_size))
            }
            false => {
                writer.write_all(self.contents.as_slice())?;

                let file_size = self.contents.len() as u32;
                Ok((file_size, file_size))
            }
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
    const WRITE_VERSION: u32 = 5;

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

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        assert!(BlockHdr::size(self.header.block_size) <= ALIGNMENT as usize);

        // Write the package header
        writer.seek(SeekFrom::Start(0))?;
        self.header.write(writer)?;

        // Calculate position of first file entry
        let block_size = self.header.block_size as usize;
        let num_blocks = (self.files.len() + block_size - 1) / block_size;
        let entries_pos = num_blocks as u64 * ALIGNMENT;

        // Write blocks with entries for all files
        let mut block_pos = self.header.first_block;
        let mut file_pos = entries_pos;
        let mut file_blocks = self.files.chunks_exact(block_size);
        for files in file_blocks.by_ref() {
            // Round up to next alignment for the block header
            let next_block = ((block_pos / ALIGNMENT) + 1) * ALIGNMENT;

            self.write_block(writer, files, block_pos, Some(next_block), &mut file_pos)?;

            block_pos = next_block;
        }

        let remainder = file_blocks.remainder();
        if !remainder.is_empty() {
            self.write_block(writer, remainder, block_pos, None, &mut file_pos)?;
        }

        Ok(())
    }

    fn write_block<W: Write + Seek>(
        &self,
        writer: &mut W,
        files: &[UOPackageFile],
        block_pos: u64,
        next_block: Option<u64>,
        file_pos: &mut u64,
    ) -> Result<()> {
        let mut headers = Vec::<FileHdr>::with_capacity(files.len());
        for file in files {
            writer.seek(SeekFrom::Start(*file_pos))?;
            let (compressed_size, raw_size) = file.write(writer, Self::WRITE_VERSION)?;

            let header_size = match Self::WRITE_VERSION {
                4 => Ok(UOPackageFile::HEADER_SIZE_V4),
                5 => Ok(UOPackageFile::HEADER_SIZE_V5),
                version => Err(Error::UnsupportedVersion(version)),
            }? as u32;

            headers.push(FileHdr {
                position: *file_pos,
                header_size,
                compressed_size,
                raw_size,
                hash: file.hash,
                _header_crc: 0,
                entry_type: file.file_type.is_compressed() as u16,
            });

            // Round up to next alignment after the written contents
            let file_pages = ((compressed_size + header_size) as u64 + ALIGNMENT - 1) / ALIGNMENT;
            *file_pos += file_pages * ALIGNMENT;
        }

        // Write the block header for the files just written
        writer.seek(SeekFrom::Start(block_pos))?;

        let header = BlockHdr {
            files_count: files.len() as u32,
            next_block: next_block.unwrap_or(0),
            headers,
        };

        header.write(writer)
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
