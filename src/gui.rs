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
    cell::RefCell
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
    miner::*,
    flightsheet,
    flightsheet::FlightSheet,
    utils,
    alloc_shared
};
//use crate::exec::RIGEL_ARGS;

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

    let selected_flightsheet = alloc_shared!(
    if !flightsheets.is_empty()
    {
        flightsheets[0].clone()
    }
    else
    {
        FlightSheet::from_gui( &gui )
    });

    update_ui_from_flightsheet( &gui, &selected_flightsheet.borrow() );
    // Populate Coin Combobox
    let coins_only: Vec<SharedString> = flightsheet::COINS
        .iter()
        .map( |c| SharedString::from( c.0 ) )
    .collect();
    gui.set_coin_list( ModelRc::new( VecModel::from( coins_only ) ) );
    // Populate Flight Sheet combobox
    let names: Vec<SharedString> = flightsheets
        .iter()
        .map( |fs| fs.name().clone().into() )
    .collect();

    println!( "names of flight sheets loaded are: {:?}", names );
    gui.set_sheets( Rc::new( VecModel::from( names ) ).into() );
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

    let state = alloc_shared!( MinerState::new() );
    {
        let weak = weak_gui.clone();
        let state = state.clone();

        gui.on_start_clicked( move ||
        {
            if let Some( gui ) = weak.upgrade()
            {
                evt_start_clicked( &gui, &state );
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
        gui.on_flightsheet_changed( move |new_sheet|
        {
            if let Some( gui ) = weak.upgrade()
            {
                evt_flightsheet_selected( &gui, new_sheet );
            }
        });
    }

    gui.run().expect( "GUI failed to execute." );

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

fn evt_start_clicked(gui: &MinerLauncher, state: &Rc<RefCell<MinerState>>)
{
    match gui.get_btn_start_txt().as_str()
    {
        "Start Miner" =>
        {
            let mut miner = state.borrow_mut();

            match miner.launch( &FlightSheet::from_gui( gui ) )
            {
                Ok( _ ) => { }
                Err( e ) => eprintln!( "Miner execution failed: {}", e )
            }
        }
        // TODO: need to fix stoppage behaviour
        "Stop Miner" =>
        {
            // let mut s = state.borrow_mut();
            // match s.stop()
            // {
            //     Ok( _ ) => gui.set_btn_start_txt( "Start Miner".into() ),
            //     Err( e ) => eprintln!( "Failed to stop miner: {}", e )
            // }
        }

        _ => { }
    }
}

fn evt_save_clicked(gui: &MinerLauncher)
{
    let mut save = FlightSheet::from_gui( gui );
    save.file = Some( env::current_dir()
        .unwrap_or( PathBuf::from( "." ) )
        .as_path()
    .join( gui.get_name() ) );

    match save.save_json()
    {
        Ok( _ ) => println!( "Save successful!" ),
        Err( e ) => eprintln!( "Failed to save flight sheet: {}", e )
    }
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

fn evt_flightsheet_selected(gui: &MinerLauncher, new_flightsheet: SharedString)// !BUG!: callback not being invoked.
{
    let flighsheet_file = PathBuf::from( new_flightsheet.as_str() );
    let dir_path = env::current_dir().unwrap_or( PathBuf::from( "." ) );
    let mut fullpath = dir_path.join( flighsheet_file.as_path() );

    if !fullpath.add_extension( "json" )
    {
        return;
    }

    println!( "{:?}", fullpath) ;

    match &FlightSheet::from_json( &fullpath )
    {
        Ok( new_flightsheet ) =>
        {
            evt_clear_clicked( gui );// Clear first to prepare for new flight sheet
            update_ui_from_flightsheet( gui, new_flightsheet );
            println!( "FlightSheet updated" );
        }

        Err( e ) => eprintln!( "Error loading flightsheet {}: {}", new_flightsheet.as_str(), e )
    }
}

pub fn update_ui_from_flightsheet(gui: &MinerLauncher, fs: &FlightSheet)
{
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
