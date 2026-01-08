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
    ComponentHandle,
    ModelRc,
    VecModel
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
    gui.set_sheets( Rc::new( VecModel::from( names ) ).into() );
    // Populate Coin Combobox
    let coins_only: Vec<SharedString> = flightsheet::COINS
        .iter()
        .map( |c| SharedString::from( c.0 ) )
    .collect();
    gui.set_coin_list( ModelRc::new( VecModel::from( coins_only ) ) );
    // Instance of a (running) miner, if it exists
    let miner = Rc::new( RefCell::new( None::<Child> ) );
    // ------------- Callbacks for events ------------------//
    {// Select Miner button
        let weak = weak_gui.clone();
        gui.on_select_miner_clicked( move ||
        {
            if let Some( gui ) = weak.upgrade()
            {
                evt_select_miner_clicked( &gui );
            }
        });
    }

    {// Start button
        let weak = weak_gui.clone();
        let miner = miner.clone();

        gui.on_start_clicked( move ||
        {
            if let Some( gui ) = weak.upgrade()
            {
                evt_start_clicked( &gui, &miner );
            }
        });
    }

    {// Save button
        let weak = weak_gui.clone();
        gui.on_save_clicked( move ||
        {
            if let Some( gui ) = weak.upgrade() 
            {
                evt_save_clicked( &gui );
            }
        });
    }

    {// Add sheet button
        let weak = weak_gui.clone();
        gui.on_add_sheet_clicked( move ||
        {
            if let Some( gui ) = weak.upgrade()
            {
                let selected_fs = selected_flightsheet.clone();

                if let Ok( new_flightsheet ) = FlightSheet::open_flightsheet()
                {
                    update_ui_from_flightsheet( &gui, &new_flightsheet );
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
            if let Some( gui ) = weak.upgrade()
            {
                evt_clear_clicked( &gui );
            }
        });
    } 

    {//ComboBox selection event for Coin
        let weak = gui.as_weak();
        gui.on_coin_changed( move |new_coin|
        {
            if let Some( gui ) = weak.upgrade()
            {
                evt_coin_selected( &gui, new_coin );
            }
        });
    }

    {// ComboBox selection event for Flightsheets
        let weak = gui.as_weak();
        gui.on_coin_changed( move |new_sheet|
        {
            if let Some( gui ) = weak.upgrade()
            {
                evt_flightsheet_selected( &gui, new_sheet );
            }
        });
    }

    gui.run().expect( "GUI failed to launch." );

    Ok(())
}
// =============================== UI Events =============================== \\
fn evt_select_miner_clicked(gui: &MinerLauncher)
{
    if let Some( miner ) = utils::select_file_dialogue( env::current_dir()
        .unwrap_or( PathBuf::from( "." ) )
        .as_path(),
        "Executable Files", &["exe"] )
    && miner.to_str().is_some()
    {
        gui.set_miner_exec( miner.to_string_lossy().to_string().into() )
    }
}

fn evt_start_clicked(gui: &MinerLauncher, miner: &Rc<RefCell<Option<Child>>>)
{
    match gui.get_btn_start_txt().as_str()
    {
        "Start Miner" =>
        {
            match FlightSheet::from_gui( gui ).launch_miner()
            {
                Ok( child ) =>
                {
                    *miner.borrow_mut() = Some( child );
                    gui.set_btn_start_txt( "Stop Miner".into() );
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
                        gui.set_btn_start_txt( "Start Miner".into() );
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

fn evt_save_clicked(gui: &MinerLauncher)
{
    let _ = FlightSheet::from_gui( gui )
        .save_json( env::current_dir()
    .unwrap_or( PathBuf::from( "." ) ).as_path() );
}

fn evt_clear_clicked(gui: &MinerLauncher)
{
    let empty = SharedString::new();
    gui.set_name( empty.clone() );
    gui.set_miner_exec( empty.clone() );
    gui.set_coin( empty.clone() );
    gui.set_wallet( empty.clone() );
    gui.set_pool( empty.clone() );
    gui.set_worker( empty.clone() );
    gui.set_core_clock( empty.clone() );
    gui.set_mem_clock( empty.clone() );
    gui.set_fan_speed( empty.clone() );
    gui.set_power_limit( empty.clone() );
    gui.set_extra_args( empty );
}

fn evt_coin_selected(gui: &MinerLauncher, new_coin: SharedString)
{
    gui.set_algorithm( flightsheet::algo_for( new_coin.to_string() ).unwrap_or_default().into() );
    gui.set_coin( new_coin );
}

fn evt_flightsheet_selected(gui: &MinerLauncher, new_flightsheet: SharedString)
{
    if let Ok( file ) = &FlightSheet::from_json( &PathBuf::from( new_flightsheet.as_str() ) )
    {
        update_ui_from_flightsheet( gui, file );
    }
}

pub fn update_ui_from_flightsheet(gui: &MinerLauncher, fs: &FlightSheet)
{
    gui.set_name( fs.name.clone().into() );
    gui.set_miner_exec( fs.miner_exec.clone().into() );
    gui.set_coin( fs.coin.clone().into() );
    gui.set_wallet( fs.wallet.clone().into() );
    gui.set_pool( fs.pool.clone().into() );

    if let Some( v ) = &fs.worker
    {
        gui.set_worker( v.clone().into() );
    }

    if let Some( v ) = &fs.core_clock
    {
        gui.set_core_clock( v.clone().into() );
    }

    if let Some( v ) = &fs.mem_clock
    {
        gui.set_mem_clock( v.clone().into() );
    }

    if let Some( v ) = &fs.fan_speed
    {
        gui.set_fan_speed( v.clone().into() );
    }

    if let Some( v ) = &fs.power_limit
    {
        gui.set_power_limit( v.clone().into() );
    }

    if let Some( v ) = &fs.extra_args
    {
        gui.set_extra_args( v.clone().into() );
    }
}
