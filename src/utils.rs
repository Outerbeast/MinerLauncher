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
    path::{ Path, PathBuf },
    process::{ Child, Command }
};
#[macro_export]
macro_rules! alloc_shared// Shared, mutable heap allocation (Rc<RefCell<T>>).
{
    ( $value:expr ) =>
    {
        std::rc::Rc::new( std::cell::RefCell::new( $value ) )
    };
}

pub fn select_file_dialogue(path: &Path, filter_name: &str, filter_list: &[&str]) -> Option<PathBuf>
{
    rfd::FileDialog::new()
        .set_directory( path )
        .add_filter( filter_name, filter_list )
    .pick_file()
}
// Executable launcher
pub fn execute(exe: &str, args: &[&str], as_admin: bool) -> io::Result<Child>
{
    let args: Vec<&str> = args
        .iter()
        .copied()
        .filter( |a| *a != exe )
    .collect();

    match as_admin
    {
        true =>
        {// ISSUE: The child process in this case is NOT the executable but the terminal that runs executing it with elevated priveledges
            Command::new( "powershell" )
                .arg( "-Command" )
                .arg
                ( format!(
                    "Start-Process '{}' -Verb RunAs -ArgumentList '{}'",
                    exe,
                    args.join( " " ) )
                )
            .spawn()
        }

        false => Command::new( exe ).args( &args ).spawn()
    }
}

pub fn stop(mut proc: Child) -> io::Result<()>
{   // Best-effort kill
    proc.kill()?;
    let _ = proc.wait()?;

    Ok(())
}

pub fn is_running(mut proc: Child) -> bool
{
    match proc.try_wait()
    {
        Ok( Some( _ ) ) => false,
        Ok( None ) => true,
        Err( _ ) => false,
    }
}

pub fn pid(proc: Child) -> Option<u32>
{
    Some( proc.id() )
}
