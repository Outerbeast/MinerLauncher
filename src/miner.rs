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
use std::
{
    io,
    path::PathBuf,
    process::
    {
        Child,
        ExitStatus
    }
};
use serde::Deserialize;
use crate::
{
    flightsheet::FlightSheet,
    utils
};

pub static ARG_SCHEMA: &str = include_str!( "argspec.json" );

pub struct ArgSpec
{
    pub flag: String,
    pub getter: fn(&FlightSheet) -> Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArgEntry
{
    pub flag: String,
    pub field: String,
}

#[derive(Debug, Deserialize)]
pub struct MinerEntry
{
    pub name: String,
    pub args: Vec<ArgEntry>,
}

pub fn get_argspecs(miner_exe: &str) -> Result<Vec<ArgSpec>, io::Error>
{   // Deserialize directly into a Vec<MinerEntry>
    let miners: Vec<MinerEntry> = serde_json::from_str( ARG_SCHEMA )?;
    // Find the miner entry
    let miner = miners.iter()
        .find( |m| m.name == miner_exe )
        .ok_or_else( || io::Error::new(
            io::ErrorKind::NotFound,
            format!( "No miner entry for '{}'", miner_exe ),
        ))?;
    // Build ArgSpec list
    let mut out = Vec::new();

    for arg in &miner.args
    {
        let getter = crate::flightsheet::getter_for( &arg.field )
            .ok_or_else( || io::Error::new(
                io::ErrorKind::InvalidData,
                format!( "Unknown field '{}' for miner '{}'", arg.field, miner_exe ),
            ))?;

        out.push( ArgSpec
        {
            flag: arg.flag.clone(),
            getter,
        });
    }

    Ok( out )
}
// Main launchery
pub enum MinerStatus
{
    Idle,
    Starting,
    Running,
    Stopped,
    Error(String),
}

pub struct MinerState
{
    pub child: Option<Child>,
    pub last_status: Option<ExitStatus>,
    pub status: MinerStatus,
}

impl Default for MinerState
{
    fn default() -> Self { Self::new() }
}
impl MinerState
{
    pub fn new() -> Self
    {
        Self
        {
            child: None,
            last_status: None,
            status: MinerStatus::Idle,
        }
    }

    pub fn launch(&mut self, flightsheet: &FlightSheet) -> Result<&mut Child, io::Error>
    {
        let exe = PathBuf::from( &flightsheet.miner_exec );
        let exe = exe
            .file_stem()
            .ok_or_else( || io::Error::new( io::ErrorKind::InvalidInput, "missing file stem" ) )?
            .to_str()
            .ok_or_else( || io::Error::new( io::ErrorKind::InvalidInput, "non-UTF8 file stem") )?;

        let specs = get_argspecs( exe )?;
        let args = flightsheet.to_args( &specs );
        let args: Vec<&str> = args.iter().map( |s| s.as_str() ).collect();

        self.status = MinerStatus::Starting;
        match utils::execute( &flightsheet.miner_exec, &args, flightsheet.needs_admin() )
        {
            Ok( kid ) =>
            {
                self.child = Some( kid );
                self.status = MinerStatus::Running;
                println!( "Miner launched with args: {:?}", args );

                let child = self.child
                    .as_mut()
                .expect( "child process must exist after successful launch" );

                Ok( child )
            }

            Err( e ) =>
            {
                self.status = MinerStatus::Error( e.to_string() );

                Err( e )
            }
        }
    }
}
