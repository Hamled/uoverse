use std::{convert::TryInto, fs::OpenOptions, io::Write};
use uoverse_tools::{
    archive::uo_package::UOPackage,
    map::{Tile, UOMap},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = "build/map6legacymul";

    let mut map = UOMap::new(256, 256)?;
    for x in 0..256 {
        for y in 0..256 {
            map.set(
                x,
                y,
                Tile {
                    kind: 0x0004,
                    height: 0x00,
                },
            )?;
        }
    }

    let files = map.into_files(prefix)?;
    let package: UOPackage = files.try_into()?;
    dbg!(&package);

    let mut package_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("map6LegacyMUL.uop")?;

    package.write(&mut package_file)?;
    package_file.flush()?;

    Ok(())
}
