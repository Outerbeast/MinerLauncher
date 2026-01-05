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
    collections::HashMap,
    path::{ Path, PathBuf },
    process::{ Child, Command }
};

use serde::{ Serialize, Deserialize };
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightSheet
{   // Required
    pub name: String,
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
}

impl FlightSheet
{    // Constructors
    pub fn from_json(file: &Path) -> Result<Self, io::Error>
    {
        let file = fs::read_to_string( file )?;
        serde_json::from_str( &file ).map_err( |e| e.into() )
    }
   
    pub fn from_gui(gui: &MinerLauncher) -> Self
    {
        Self
        {
            name: gui.get_name().to_string(),
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
    pub fn to_args(&self) -> Vec<String>
    {   // Format: "C:\path\to\miner.exe --coin <coin> --algorithm <algo> --username <wallet> --url <pool>"
        let mut args = vec!
        [// !-TODO-!: Support for other miner argument schemes- currently Rigel only
            self.miner_exec.clone(),
            "--coin".into(), self.coin.clone(),
            "--algorithm".into(), algo_for( self.coin.clone() ).expect( "Algorithm for coin is missing!" ).to_string(),
            "--username".into(), self.wallet.clone(),
            "--url".into(), self.pool.clone(),
        ];

        if let Some( worker ) = &self.worker
        {
            args.push( "--worker".into() );
            args.push( worker.clone() );
        }

        if let Some( core ) = &self.core_clock
        {
            args.push( "--cclock".into() );
            args.push( core.clone() );
        }

        if let Some( mem ) = &self.mem_clock
        {
            args.push( "--mclock".into() );
            args.push( mem.clone() );
        }

        if let Some( fan ) = &self.fan_speed
        {
            args.push( "--fan-control".into() );
            args.push( fan.clone() );
        }

        if let Some( pl ) = &self.power_limit
        {
            args.push( "--pl".into() );
            args.push( pl.clone() );
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
            None => Err( io::Error::new( io::ErrorKind::NotFound, "No file selected." ) ),
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
            if path.extension().and_then( |s| s.to_str() ) == Some( "json" )
            && let Ok( sheet ) = Self::from_json( &path )
            {
                sheets.push( sheet );
            }
        }

        Ok( sheets )
    }

    pub fn save_json(&self, path: &Path) -> Result<(), io::Error>
    {
        let json = serde_json::to_string_pretty( self )?;
        fs::write( path, json )?;

        Ok(())
    }
    // Setting GPU tunes requires elevated perms
    fn needs_admin(&self) -> bool
    {   
        self.core_clock.is_some()
        || self.mem_clock.is_some()
        || self.fan_speed.is_some()
        || self.power_limit.is_some()
    }

    pub fn launch_miner(&self) -> Result<Child, io::Error>
    {
        if self.miner_exec.is_empty() || !self.miner_exec.ends_with( ".exe" )
        {
            return Err( io::Error::new( io::ErrorKind::InvalidInput, "Miner tool executable not set." ) );
        }

        match self.needs_admin()
        {
            true => self.launch_admin(),
            false => self.launch_normal()
        }
    }

    fn launch_normal(&self) -> Result<Child, io::Error>
    {
        let mut args = self.to_args();
        println!( "Launching miner with arguments {}", args.join( " " ) );
        if args.is_empty()
        {
            return Err( io::Error::new( io::ErrorKind::InvalidInput, "No program specified." ) );
        }

        let program = args.remove( 0 );
        Command::new( program ).args( args ).spawn()
    }

    fn launch_admin(&self) -> Result<Child, io::Error>
    {
        let mut args = self.to_args();
        println!( "Launching admin miner with arguments {}", args.join( " " ) );
        let program = args.remove(0);

        Command::new( "powershell" )
            .arg( "-Command" )
            .arg( format!(
                "Start-Process '{}' -ArgumentList '{}' -Verb runAs",
                program,
                args.join( " " )
            ))
        .spawn()
    }

    #[allow( dead_code )]
    pub fn stop_miner(&self) -> Result<Child, io::Error>
    {
        let processname = self.miner_exec.clone();
        println!( "Stopping miner:{}", processname );
        Command::new( "powershell" )
            .arg( "-Command" )
            .arg( format!( "Get-Process | Where-Object ProcessName -like {} | Stop-Process", processname ) )
        .spawn()
    }
}

#[derive(Debug, Deserialize)]
pub struct ArgSpec
{
    pub flag: String,
    pub format: String,
}

pub type MinerSchema = HashMap<String, HashMap<String, ArgSpec>>;

pub fn load_schema(path: &Path) -> MinerSchema
{
    let content = std::fs::read_to_string( path )
        .expect( "miners.toml missing" );

    toml::from_str( &content )
        .expect( "invalid miners.toml" )
}

pub fn algo_for(coin: String) -> Option<&'static str>
{
    match coin.to_lowercase().as_str()
    {   // Autolykos2
        "ergo"        => Some( "autolykos2" ),
        "erg"         => Some( "autolykos2" ),
        // Ethash family
        "etc"         => Some( "ethash" ),
        "ethw"        => Some( "ethash" ),
        "etho"        => Some( "ethash" ),
        // KawPow family
        "rvn"         => Some( "kawpow" ),
        "ravencoin"   => Some( "kawpow" ),
        "neoxa"       => Some( "kawpow" ),
        "neurai"      => Some( "kawpow" ),
        "quai"        => Some( "kawpow" ),
        "mew"         => Some( "kawpow" ),
        "meowcoin"    => Some( "kawpow" ),
        // ZelHash / FluxHash
        "flux"        => Some( "zelhash" ),
        // BeamHashIII
        "beam"        => Some( "beamhash" ),
        // FiroPoW
        "firo"        => Some( "firopow" ),
        // ProgPoW variants
        "sero"        => Some( "progpow" ),
        "veil"        => Some( "progpow" ),
        "epic"        => Some( "progpow-epic" ),
        // Octopus (Conflux)
        "cfx"         => Some( "octopus" ),
        // kHeavyHash (Kaspa)
        "kas"         => Some( "kheavyhash" ),
        "kaspa"       => Some( "kheavyhash" ),
        // Blake3-based
        "iron"        => Some( "blake3" ),
        "alph"        => Some( "blake3-alph" ),
        // DynexSolve
        "dnx"         => Some( "dynexsolve" ),
        // KarlsenHash
        "ksl"         => Some( "karlsenhash" ),
        "karlsen"     => Some( "karlsenhash" ),
        // NexaPoW
        "nexa"        => Some( "nexapow" ),

        _ => None,
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
