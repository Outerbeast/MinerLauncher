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
    process::
    {
        Command,
        Child,
        ExitStatus
    }
};

pub fn select_file_dialogue(path: &Path, filter_name: &str, filter_list: &[&str]) -> Option<PathBuf>
{
    rfd::FileDialog::new()
        .set_directory( path )
        .add_filter( filter_name, filter_list )
    .pick_file()
}

pub fn exec_program(mut args: Vec<String>) -> Result<Child, io::Error>
{
    if args.is_empty()
    {
        return Err( io::Error::new( io::ErrorKind::InvalidInput, "No program specified." ) );
    }

    let program = args.remove( 0 );

    Command::new( program ).args( args ).spawn()
}

pub fn stop_process(mut proc: Child) -> Result<ExitStatus, io::Error>
{
    proc.kill().ok();
    proc.wait()
}

/* use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Threading::*,
    Win32::UI::Shell::*,
};

pub struct ElevatedProcess {
    pub handle: HANDLE,
    pub pid: u32,
}

pub fn exec_elevated(program: &str, args: &str) -> Result<ElevatedProcess>
{
    unsafe {
        let verb = w!("runas");
        let file = HSTRING::from(program);
        let params = HSTRING::from(args);

        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            lpVerb: verb,
            lpFile: PWSTR(file.as_ptr() as *mut _),
            lpParameters: PWSTR(params.as_ptr() as *mut _),
            nShow: SW_SHOW,
            ..Default::default()
        };

        if !ShellExecuteExW(&mut sei).as_bool() {
            return Err(Error::from_win32());
        }

        if sei.hProcess.is_invalid() {
            return Err(Error::new(E_FAIL, "No process handle returned".into()));
        }

        let pid = GetProcessId(sei.hProcess);

        Ok(ElevatedProcess {
            handle: sei.hProcess,
            pid,
        })
    }
}
 */