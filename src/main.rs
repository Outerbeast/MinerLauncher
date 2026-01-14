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
//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

pub mod gui;
pub mod flightsheet;
pub mod utils;
pub mod exec;

use std::
{
    env,
    io,
    path::PathBuf
};

use crate::{ flightsheet::FlightSheet };
use crate::exec::MinerState;
use crate::flightsheet::RIGEL_ARGS;

slint::include_modules!();

fn run() -> Result<(), io::Error>
{
    let args: Vec<String> = env::args().collect();

    match args.len()
    {
        n if n > 1 =>
        {
            if args[1].ends_with( ".json" )
            {
                let fs = FlightSheet::from_json( &PathBuf::from( &args[1] ) )?;
                //let _ = FlightSheet::launch_miner( &fs );
                let args = fs.to_args( RIGEL_ARGS );
                let args: Vec<&str> = args.iter().map( |s| s.as_str() ).collect();
                MinerState::launch( &mut MinerState::new(), fs.miner_exec.as_str(), &args, fs.needs_admin() )?;
            }
        }

        _ => gui::launch_gui()?
    }

    Ok(())
}

fn main() -> std::process::ExitCode
{
    match run()
    {
        Ok( _ ) =>
        {
            println!( "Application ran successfully." );
            std::process::ExitCode::SUCCESS
        }

        Err( e ) =>
        {
            eprintln!( "Application error: {}", e );
            std::process::ExitCode::FAILURE
        }
    }
}
