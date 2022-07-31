#![feature(io_error_more)]

use std::{env::args, fs, io, path};
use uoverse_tools::archive::uo_package::UOPackage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if args().len() > 1 {
        let path_arg = args().last().unwrap();
        let package_path = path::Path::new(&path_arg);
        let package_name = package_path
            .file_name()
            .ok_or(io::Error::new(
                io::ErrorKind::IsADirectory,
                "Path must be a UOP file",
            ))?
            .to_str()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "UOP file path was invalid",
            ))?;

        let mut package_file = fs::OpenOptions::new().read(true).open(package_path)?;
        let package = UOPackage::new(&mut package_file)?;
        dbg!(&package);

        let dir_name = format!("{}_unpack", package_name);
        fs::create_dir(dir_name.as_str())?;
        for file in package.files {
            fs::write(
                format!("./{}/{:016X}.dat", dir_name, file.hash),
                file.contents.as_slice(),
            )?;
        }
    }

    Ok(())
}
