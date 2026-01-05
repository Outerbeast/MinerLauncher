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
    env,
    io,
    path::PathBuf,
    rc::Rc,
    cell::RefCell,
    process::Child
};

use slint::
{
    SharedString,
    ComponentHandle
};

use crate::
{
    MinerLauncher,
    flightsheet,
    flightsheet::FlightSheet,
    utils
};
// Build UI
pub fn launch_gui() -> Result<(), io::Error>
{
    let gui = MinerLauncher::new().expect( "Failed to initialise GUI." );
    // Create a weak handle for callbacks
    let weak_gui = gui.as_weak();
    // Load flightsheets
    let flightsheets = 
    FlightSheet::load_flightsheets
    (
        env::current_dir()
        .unwrap_or( PathBuf::from( "." ) )
        .as_path()
    ).unwrap_or_default();

    let selected_flightsheet = Rc::new( RefCell::new(
    if !flightsheets.is_empty()
    {
        flightsheets[0].clone()
    }
    else
    {
        FlightSheet::from_gui( &gui )
    }));

    update_ui_from_flightsheet( &gui, &selected_flightsheet.borrow() );
    // Populate Flight Sheet combobox
    let names: Vec<slint::SharedString> = flightsheets
        .iter()
        .map( |fs| fs.name.clone().into() )
    .collect();

    let model = Rc::new( slint::VecModel::from( names ) );
    gui.set_sheets( model.into() );
    // Instance of a (running) miner, if it exists
    let miner = Rc::new( RefCell::new( None::<Child> ) );
    // ------------- Callbacks for events ------------------//
    {// Select Miner button
        let weak = weak_gui.clone();
        gui.on_select_miner_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade()
            {
                evt_select_miner_clicked( &ui );
            }
        });
    }

    {// Start button
        let weak = weak_gui.clone();
        let miner = miner.clone();

        gui.on_start_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade()
            {
                evt_start_clicked( &ui, &miner );
            }
        });
    }

    {// Save button
        let weak = weak_gui.clone();
        gui.on_save_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade() 
            {
                evt_save_clicked( &ui );
            }
        });
    }

    {// Add sheet button
        let weak = weak_gui.clone();
        gui.on_add_sheet_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade()
            {
                let selected_fs = selected_flightsheet.clone();

                if let Ok( new_flightsheet ) = FlightSheet::open_flightsheet()
                {
                    update_ui_from_flightsheet( &ui, &new_flightsheet );
                    *selected_fs.borrow_mut() = new_flightsheet;
                    // !-TODO-!: update flight sheet combobox
                }
            }
        });
    }
    
    {// Clear button
        let weak = weak_gui.clone();
        gui.on_remove_sheet_clicked( move ||
        {
            if let Some( ui ) = weak.upgrade()
            {
                evt_clear_clicked( &ui );
            }
        });
    } 

    {//ComboBox selection event for Coin - This might not be working properly.
        let weak = gui.as_weak();
        gui.on_coin_changed( move |new_coin|
        {
            if let Some( ui ) = weak.upgrade()
            {
                evt_coin_selected( &ui, new_coin );
            }
        });
    }

    gui.run().expect( "GUI failed to launch." );

    Ok(())
}
// =============================== UI Events =============================== \\
fn evt_select_miner_clicked(ui: &MinerLauncher)
{
    match utils::select_file_dialogue( env::current_dir()
        .unwrap_or( PathBuf::from( "." ) )
        .as_path(),
        "Executable Files", &["exe"] )
    {
        Some( miner ) => 
        {
            if miner.to_str().is_some()
            {
                ui.set_miner_exec( miner.to_string_lossy().to_string().into() )
            }
        }
        // Don't change anything
        None => { }
    }
}

fn evt_start_clicked(ui: &MinerLauncher, miner: &Rc<RefCell<Option<Child>>>)
{
    match ui.get_btn_start_txt().as_str()
    {
        "Start Miner" =>
        {
            match FlightSheet::from_gui( ui ).launch_miner()
            {
                Ok( child ) =>
                {
                    *miner.borrow_mut() = Some( child );
                    ui.set_btn_start_txt( "Stop Miner".into() );
                }

                Err( _e ) => { }
            }
        }

        "Stop Miner" =>
        {   // Take the child out in a single borrow
            let child =
            {
                let mut borrow = miner.borrow_mut();
                borrow.take() // Option::take() replaces with None and returns the Child
            };
            // Now the borrow is dropped automatically (end of block)
            if let Some( mut child ) = child
            {
                match child.kill()
                {
                    Ok( _ ) =>
                    {
                        ui.set_btn_start_txt( "Start Miner".into() );
                    }

                    Err( e ) =>
                    {
                        eprintln!( "Failed to stop process: {}", e );
                        // If kill failed, you may want to put the child back:
                        // *miner.borrow_mut() = Some(child);
                    }
                }
            }
        }

        _ => { }

    }
}

fn evt_save_clicked(ui: &MinerLauncher)
{
    let _ = FlightSheet::from_gui( &ui )
    .save_json( env::current_dir()
    .unwrap_or( PathBuf::from( "." ) ).as_path() );
}

fn evt_clear_clicked(ui: &MinerLauncher)
{
    let empty = SharedString::new();
    ui.set_name( empty.clone() );
    ui.set_miner_exec( empty.clone() );
    ui.set_coin( empty.clone() );
    ui.set_wallet( empty.clone() );
    ui.set_pool( empty.clone() );
    ui.set_worker( empty.clone() );
    ui.set_core_clock( empty.clone() );
    ui.set_mem_clock( empty.clone() );
    ui.set_fan_speed( empty.clone() );
    ui.set_power_limit( empty.clone() );
    ui.set_extra_args( empty );
}

fn evt_coin_selected(ui: &MinerLauncher, new_coin: SharedString)
{
    let algo = flightsheet::algo_for( new_coin.to_string() );
    ui.set_algorithm( algo.unwrap_or_default().into() );
    println!( "Algorithm is now set to: {:?}", algo.clone() );
}

pub fn update_ui_from_flightsheet(ui: &MinerLauncher, fs: &FlightSheet)
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
