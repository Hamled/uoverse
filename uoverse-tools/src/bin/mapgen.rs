use std::{fs, io::Cursor};
use uoverse_tools::archive::uo_package::UOPackage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read("/home/charles/games/uo/client-classic/MainMisc.uop")?;

    let mut reader = Cursor::new(contents.as_slice());
    let package = UOPackage::new(&mut reader)?;

    dbg!(&package);
    for file in package.files {
        fs::write(format!("./{:016X}.dat", file.hash), file.contents)?;
    }

    Ok(())
}
