use crate::statics::TRACER_RELOAD_HANDLE;
use std::ptr;
use std::sync::{Mutex, OnceLock};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_chrome::{ChromeLayerBuilder, FlushGuard};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, fmt, reload, EnvFilter, Layer};
use windows_sys::core::PCSTR;
use windows_sys::Win32::Foundation::{GENERIC_WRITE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileA, OPEN_EXISTING};
use windows_sys::Win32::System::Console::{AllocConsole, FreeConsole, GetConsoleWindow};
use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOW};

static FILE_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
static CHROME_GUARD: OnceLock<Mutex<FlushGuard>> = OnceLock::new();

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
            let ansi_support_enabled = nu_ansi_term::enable_ansi_support().is_ok();

            // Initialize the tracing subscriber with both console and file logging
            let file_appender = RollingFileAppender::new(Rotation::NEVER, ".", "log_output.log");
            let (non_blocking_file_appender, file_guard) =
                tracing_appender::non_blocking(file_appender);

            let (chrome_layer, chrome_guard) = ChromeLayerBuilder::new()
                .include_args(true)
                .build();

            let (tracer_filter, reload_handle) =
                reload::Layer::new(Targets::new().with_target("tracer", Level::ERROR));
            let _ = FILE_GUARD.set(file_guard);
            let _ = CHROME_GUARD.set(Mutex::new(chrome_guard));
            let _ = TRACER_RELOAD_HANDLE.set(reload_handle);
            
            let env_filter = EnvFilter::from_default_env().add_directive(Level::INFO.into());

            let fmt_layer = fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(ansi_support_enabled)
                .with_timer(ChronoLocal::new("%H:%M:%S%.3f".into()))
                .with_span_events(FmtSpan::CLOSE)
                .with_level(true);

            let file_layer = fmt::layer()
                .with_writer(non_blocking_file_appender)
                .with_ansi(false)
                .with_timer(ChronoLocal::rfc_3339())
                .with_span_events(FmtSpan::ENTER)
                .with_level(true);
            
            let normal_layers =  fmt_layer
                .and_then(file_layer)
                .with_filter(env_filter)
                .with_filter(filter::filter_fn(|metadata| metadata.target() != "tracer"));
            
            let tracer_layer = chrome_layer.with_filter(tracer_filter);

            tracing_subscriber::registry()
                .with(tracer_layer)
                .with(normal_layers)
                .init();
        } else {
            panic!("Unable to get console handle!");
        }
    }
}
