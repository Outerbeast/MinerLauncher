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
    path::{ Path, PathBuf }
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
