use std::thread;
use std::thread::{sleep, Thread};
use std::time::Duration;
use tracing::info;
use windows_sys::Win32::Foundation::{BOOL, HINSTANCE};
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use crate::console::open_console;

mod console;

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
                open_console(); // You can call the function to open a console here

                info!("Attached!");

                loop {
                    sleep(Duration::from_secs(1));
                    info!("A second passed.");
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