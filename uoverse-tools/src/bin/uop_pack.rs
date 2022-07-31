use std::{
    convert::TryInto,
    env::args,
    fs::{self, OpenOptions},
    io::Write,
};
use uoverse_tools::archive::uo_package::{uop_hash, FileType, UOPackage, UOPackageFile};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let mut args = args();
    if args.len() < 3 {
        println!(
            "Usage: {} <package path> <file to include ...>",
            args.next().unwrap()
        );

        return Ok(());
    }

    let mut args = args.skip(1);
    let package_path = args.next().unwrap();
    let file_paths: Vec<String> = args.collect();

    let mut files = Vec::<UOPackageFile>::with_capacity(file_paths.len());
    for path in file_paths {
        files.push(UOPackageFile {
            hash: uop_hash(path.as_str())?,
            file_type: FileType::Compressed,
            timestamp: None,
            contents: fs::read(path)?,
        });
    }

    let package: UOPackage = files.try_into()?;
    dbg!(&package);

    let mut package_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(package_path)?;

    package.write(&mut package_file)?;
    package_file.flush()?;

    Ok(())
}
