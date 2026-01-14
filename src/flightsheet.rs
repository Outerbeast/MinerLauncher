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
use crate::MinerLauncher;

use std::
{
    env,
    fs,
    io,
    path::{ Path, PathBuf }
};

use serde::{ Serialize, Deserialize };
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightSheet
{   // Required
    #[serde(rename = "exe")]
    pub miner_exec: String,

    pub coin: String,
    pub wallet: String,
    pub pool: String,
    // Optional / nullable
    pub worker: Option<String>,

    #[serde(rename = "core")]
    pub core_clock: Option<String>,

    #[serde(rename = "mem")]
    pub mem_clock: Option<String>,

    #[serde(rename = "fan")]
    pub fan_speed: Option<String>,

    #[serde(rename = "pl")]
    pub power_limit: Option<String>,

    pub extra_args: Option<String>,
    pub file: Option<PathBuf>,
}

impl FlightSheet
{    // Constructors
    pub fn from_json(path: &Path) -> Result<Self, io::Error>
    {
        let contents = fs::read_to_string(path)?;
        let mut sheet: Self = serde_json::from_str( &contents )?;
        // Inject the full path into the struct
        sheet.file = Some( path.to_path_buf() );

        println!( "Successfully loaded flightsheet file: {}", path.display() );

        Ok( sheet )
    }
    // Name of the flightsheet (from flightsheet filename)
    pub fn name(&self) -> String
    {
        match &self.file
        {
            Some( f ) =>
            {
                println!( "FlightSheet::name : {}", f.display() );
                f.file_stem().and_then( |s| s.to_str() ).unwrap_or_default().to_string()
            }

            None =>
            {
                eprintln!( "FlightSheet::name : file name is empty.");
                "".to_string()
            }
        }
    }

    pub fn from_gui(gui: &MinerLauncher) -> Self
    {
        Self
        {
            file: None,
            miner_exec: gui.get_miner_exec().to_string(),
            coin: gui.get_coin().to_string(),
            wallet: gui.get_wallet().to_string(),
            worker: opt( gui.get_worker() ),
            pool: gui.get_pool().to_string(),
            core_clock: opt( gui.get_core_clock() ),
            mem_clock: opt( gui.get_mem_clock() ),
            fan_speed: opt( gui.get_fan_speed() ),
            power_limit: opt( gui.get_power_limit() ),
            extra_args: opt( gui.get_extra_args() ),
        }
    }
    // Methods
    pub fn to_args(&self, schema: &[ArgSpec]) -> Vec<String>
    {
        let mut args = vec![self.miner_exec.clone()];

        for spec in schema
        {
            if let Some(value) = ( spec.getter )( self )
            {
                args.push( spec.flag.into() );
                args.push( value );
            }
        }

        if let Some( extra ) = &self.extra_args
        && !extra.trim().is_empty()
        {
            args.extend( extra.split_whitespace().map( |s| s.to_string() ) );
        }

        args
    }

    pub fn open_flightsheet() -> Result<Self, io::Error>
    {
        match utils::select_file_dialogue( env::current_dir().unwrap_or( PathBuf::from( "." ) ).as_path(),
            "JSON Files", &["json"] )
        {
            Some( path ) => FlightSheet::from_json( &path ),
            None => Err( io::Error::new( io::ErrorKind::NotFound, "No file selected." ) )
        }
    }

    pub fn load_flightsheets(path: &Path) -> Result<Vec<Self>, io::Error>
    {
        let mut sheets = Vec::new();

        for entry in fs::read_dir( path )?
        {
            let entry = entry?;
            let path: PathBuf = entry.path();
            // Only process *.json files
            if path.extension().and_then(|s| s.to_str()) == Some("json")
            {
                match Self::from_json(&path)
                {
                    Ok( sheet ) => sheets.push( sheet ),
                    Err( e ) => println!( "Failed to parse JSON {:?}: {}", path, e )
                }
            }
            else
            {
                println!("Skipping non-json file: {:?}", path);
            }

        }

        Ok( sheets )
    }

    pub fn save_json(&self) -> Result<(), io::Error>
    {
        let json = serde_json::to_string_pretty( self )?;
        fs::write( self.file.as_ref().unwrap().clone(), json )?;

        Ok(())
    }
    // Setting GPU tunes requires elevated perms
    pub fn needs_admin(&self) -> bool
    {   
        self.core_clock.is_some()
        || self.mem_clock.is_some()
        || self.fan_speed.is_some()
        || self.power_limit.is_some()
    }
}

#[inline]
fn opt(s: slint::SharedString) -> Option<String>
{
    let s = s.trim();
    match s.is_empty()
    {
        true => None,
        false => Some( s.to_string() ),
    }
}

pub const COINS: &[(&str, &str)] = 
&[
    // ethash family
    ("etc", "etchash"),
    ("exp", "ethash"),
    ("clo", "ethash"),
    ("ubq", "ubqhash"),
    // autolykos
    ("erg", "autolykos2"),
    // kawpow
    ("rvn", "kawpow"),
    ("neox", "kawpow"),
    ("rne", "kawpow"),
    ("quai", "kawpow" ),
    ("mewc", "kawpow"),
    ("xna", "kawpow"),
    // kheavyhash (kaspa)
    ("kas", "kheavyhash"),
    // nexa
    ("nexa", "nexapow"),
    // beam
    ("beam", "beamhashiii"),
    // equihash family
    ("zec", "equihash 200,9"),
    ("zen", "equihash 144,5"),
    ("btg", "equihash 144,5"),
    ("zcl", "equihash 192,7"),
    // cuckoo / cuckatoo / cuckaroo
    ("grin", "cuckatoo32"),
    ("mwc", "cuckaroo29"),
    ("aion", "cuckoo29"),
    // octopus
    ("cfx", "octopus"),
    // verthash
    ("vtc", "verthash"),
    // progpow variants
    ("sero", "progpow"),
    ("epic", "progpow"),
    ("firo", "firopow"),
    // randomx (cpu)
    ("xmr", "randomx"),
    ("wownero", "randomx"),
    ("dero", "randomx"),
    // yespower family
    ("rtm", "yespowerrtm"),
    ("ytn", "yespower"),
    ("xmy", "yespower"),
    // argon2 family
    ("arq", "argon2id"),
    ("xla", "argon2id"),
    ("ufo", "argon2d"),
    // ghostrider
    ("rtm", "ghostrider"),
    // ironfish
    ("iron", "fishhash"),
    // alephium
    ("alph", "blake3pow"),
    // handshake
    ("hns", "blake2b+sha3"),
    // zilliqa (dual mining)
    ("zil", "ethash"),
    // dynex
    ("dnx", "dynexsolve"),
    // radiant
    ("rxd", "sha512_256d"),
    // pyrin
    ("pyi", "pyrinhash")
];

pub fn algo_for(coin: String) -> Option<&'static str>
{
    Some( COINS.iter().find( |c| c.0 == coin )?.1 )
}

pub struct ArgSpec
{
    pub flag: &'static str,
    pub getter: fn(&FlightSheet) -> Option<String>,
}

pub const RIGEL_ARGS: &[ArgSpec] =
    &[
    ArgSpec
    {
        flag: "--coin",
        getter: |fs| Some(fs.coin.clone()),
    },
    ArgSpec
    {
        flag: "--algorithm",
        getter: |fs| algo_for( fs.coin.clone() ).map( |a| a.to_string() ),
    },
    ArgSpec
    {
        flag: "--username",
        getter: |fs| Some(fs.wallet.clone()),
    },
    ArgSpec
    {
        flag: "--url",
        getter: |fs| Some(fs.pool.clone()),
    },
    // Optional
    ArgSpec
    {
        flag: "--worker",
        getter: |fs| fs.worker.clone(),
    },
    ArgSpec
    {
        flag: "--cclock",
        getter: |fs| fs.core_clock.clone(),
    },
    ArgSpec
    {
        flag: "--mclock",
        getter: |fs| fs.mem_clock.clone(),
    },
    ArgSpec
    {
        flag: "--fan-control",
        getter: |fs| fs.fan_speed.clone(),
    },
    ArgSpec
    {
        flag: "--pl",
        getter: |fs| fs.power_limit.clone(),
    },
];
