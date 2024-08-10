use std::ptr;

use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use windows_sys::core::PCSTR;
use windows_sys::Win32::Foundation::{GENERIC_WRITE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileA, OPEN_EXISTING};
use windows_sys::Win32::System::Console::{AllocConsole, FreeConsole, GetConsoleWindow};
use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOW};

pub fn open_console() {
    unsafe {
        // Show the existing console window
        ShowWindow(GetConsoleWindow(), SW_SHOW);

        // Free and allocate a new console
        FreeConsole();
        AllocConsole();

        // Redirect stdout and stderr to the console
        let handle = CreateFileA(
            b"CONOUT$\0".as_ptr() as PCSTR,
            GENERIC_WRITE,
            0,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        );

        if handle != INVALID_HANDLE_VALUE {
            let enable_ansi = nu_ansi_term::enable_ansi_support().is_ok();

            // Initialize the tracing subscriber with both console and file logging
            let file_appender = RollingFileAppender::new(Rotation::NEVER, ".", "log_output.log");

            let fmt_layer = fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(enable_ansi)
                .with_timer(ChronoLocal::new("%H:%M:%S%.3f".into()))
                .with_span_events(FmtSpan::CLOSE)
                .with_level(true);

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_timer(ChronoLocal::rfc_3339())
                .with_span_events(FmtSpan::CLOSE)
                .with_level(true);

            let env_filter = EnvFilter::from_default_env()
                .add_directive(Level::INFO.into());

            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(file_layer)
                .with(env_filter)
                .init();
        }
    }
}