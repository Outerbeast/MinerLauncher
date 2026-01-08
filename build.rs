/*
	Miner Launcher

Copyright (C) 2026 Outerbeast
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
*/
#[cfg(windows)]
fn main() -> std::io::Result<()>
{
    let path = format!( "{}/ui/MinerLauncher.slint", env!( "CARGO_MANIFEST_DIR" ) );
    slint_build::compile( path ).map_err( std::io::Error::other )?;

    winresource::WindowsResource::new()
        .set_icon( "ui/icon.ico" )
        .set( "ProductName", "MinerLauncher" )
        .set( "ProductVersion", "1.0.0" )
        .set( "FileDescription", "Miner Launcher" )
        .set( "FileVersion", "1.0.0" )
        .set( "LegalCopyright", "Outerbeast" )
        .set( "OriginalFilename", "MinerLauncher.exe" )
        .set( "InternalName", "MinerLauncher" )
        .set( "CompanyName", "Outerbeast" )
        .set( "LegalTrademarks", "Outerbeast" )
        .set( "Comments", "Miner Launcher" )
    .compile()?;

    Ok(())
}

#[cfg(not(windows))]
fn main() -> std::io::Result<()>
{
    compile_error!( "This application only supports Windows targets" );
}