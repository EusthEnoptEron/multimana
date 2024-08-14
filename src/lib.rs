use std::thread;

use tracing::error;
use windows_sys::Win32::Foundation::{BOOL, HINSTANCE};
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use crate::console::open_console;
use crate::setup::setup;

mod console;
mod setup;
mod multiplayer;
mod utils;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    _hinst_dll: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: usize,
) -> BOOL {
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            thread::spawn(|| {
                let panics = std::panic::catch_unwind(|| {
                    open_console(); // You can call the function to open a console here

                    if let Err(e) = setup() {
                        error!("Error happened: {:?}", e);
                    }
                });

                if let Err(something) = panics {
                    error!("A fatal error occurred: {:?}", something);
                }
            });
        }
        DLL_PROCESS_DETACH => {
            // Perform actions needed when the DLL is unloaded
        }
        _ => {}
    }
    1 // Return TRUE to indicate success
}
