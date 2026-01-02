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

pub mod flightsheet;
pub mod utils;

use std::
{
    rc::Rc,
    cell::RefCell,
    env,
    io,
    path::PathBuf
};

use crate::flightsheet::FlightSheet;

slint::include_modules!();

fn launch_gui() -> Result<(), io::Error>
{
    let ui = MinerLauncher::new().expect( "PANIC!!!!!! Something bad happened" );
    // Create a weak handle for callbacks
    let weak_ui = ui.as_weak();
    // Load flightsheets
    let flightsheets = FlightSheet::load_flightsheets(
        env::current_dir().expect( "PANIC!!!!!! Something bad happened" ).as_path()
    ).expect( "PANIC!!!!!! Something bad happened" );

    let selected_flightsheet = Rc::new( RefCell::new(
    if !flightsheets.is_empty()
    {
        flightsheets[0].clone()
    }
    else
    {
        FlightSheet::from_gui( &ui )
    }));

    update_ui_from_flightsheet( &ui, &selected_flightsheet.borrow() );
    // Populate Flight Sheet combobox
    let names: Vec<slint::SharedString> = flightsheets
        .iter()
        .map( |fs| fs.name.clone().into() )
    .collect();

    let model = Rc::new( slint::VecModel::from( names ) );
    ui.set_sheets( model.into() );
    // ------------- Callbacks for events ------------------//
    {// Select Miner button
        let weak = weak_ui.clone();
        ui.on_select_miner_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade()
            {
                match pick_miner()
                {
                    Some( miner ) => 
                    {
                        if miner.to_str().is_some()
                        {
                            ui.set_miner_exec( miner.to_string_lossy().to_string().into() )
                        }
                    },

                    None => { popup( &ui, "No miner was selected." ); }
                }
            }
        });
    }

    {// Start button
        let weak = weak_ui.clone();
        ui.on_start_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade()
            {
                let _ = FlightSheet::from_gui( &ui ).launch_miner();
            }
        });
    }

    {// Save button
        let weak = weak_ui.clone();
        ui.on_save_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade() 
            {
                let _ = FlightSheet::from_gui( &ui ).save_json( env::current_dir().expect( "PANIC!!!!!! Something bad happened" ).as_path() );
            }
        });
    }

    {// Add sheet
        let weak = weak_ui.clone();
        let selected_fs = selected_flightsheet.clone();

        ui.on_add_sheet_clicked( move ||
        {
            if let Ok( new_flightsheet ) = add_flightsheet()
            {
                if let Some( ui ) = weak.upgrade()
                {
                    update_ui_from_flightsheet( &ui, &new_flightsheet );
                }

                *selected_fs.borrow_mut() = new_flightsheet;
            }
        });
    }
    
    {// Remove sheet
        let weak = weak_ui.clone();
        ui.on_remove_sheet_clicked( move ||
        {
            if let Some( _ui ) = weak.upgrade()
            {
                todo!()
            }
        });
    }

    {//ComboBox selection event - This might not be working properly.
        let weak = ui.as_weak();
        ui.on_coin_changed( move |new_coin|
        {
            if let Some( ui ) = weak.upgrade()
            {
                let algo = flightsheet::algo_for( new_coin.to_string() );
                ui.set_algorithm( algo.unwrap_or_default().into() );
                println!( "Algorithm is now set to: {:?}", algo.clone() );
            }
        });
    }

    ui.run().expect( "PANIC!!!!!! Something bad happened" );

    Ok(())
}

fn pick_miner() -> Option<PathBuf>
{
    utils::select_file_dialogue( env::current_dir().unwrap_or( PathBuf::from( "." ) ).as_path(), 
    "Executable Files", &["exe"] )
}

fn add_flightsheet() -> Result<FlightSheet, io::Error>
{
    match utils::select_file_dialogue( env::current_dir().unwrap_or( PathBuf::from( "." ) ).as_path(),
        "JSON Files", &["json"] )
    {
        Some( path ) => FlightSheet::from_json( &path ),
        None => Err( io::Error::new( io::ErrorKind::NotFound, "No file selected." ) ),
    }
}

fn popup(ui: &MinerLauncher, msg: &str)
{
    ui.set_popup_message( msg.into() );
    ui.invoke_show_popup();
}

fn update_ui_from_flightsheet(ui: &MinerLauncher, fs: &FlightSheet)
{
    ui.set_name( fs.name.clone().into() );
    ui.set_miner_exec( fs.miner_exec.clone().into() );
    ui.set_coin( fs.coin.clone().into() );
    ui.set_wallet( fs.wallet.clone().into() );
    ui.set_pool( fs.pool.clone().into() );

    if let Some( v ) = &fs.worker
    {
        ui.set_worker( v.clone().into() );
    }

    if let Some( v ) = &fs.core_clock
    {
        ui.set_core_clock( v.clone().into() );
    }

    if let Some( v ) = &fs.mem_clock
    {
        ui.set_mem_clock( v.clone().into() );
    }

    if let Some( v ) = &fs.fan_speed
    {
        ui.set_fan_speed( v.clone().into() );
    }

    if let Some( v ) = &fs.power_limit
    {
        ui.set_power_limit( v.clone().into() );
    }

    if let Some( v ) = &fs.extra_args
    {
        ui.set_extra_args( v.clone().into() );
    }
}

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
                let _ = FlightSheet::launch_miner( &fs );
            }
        }

        _ => launch_gui()?,
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
