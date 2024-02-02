#[cfg(target_os = "wasi")]
mod unix;
#[cfg(target_os = "wasi")]
pub use unix::FileBackend;

#[cfg(unix)]
mod rustix_be;
#[cfg(unix)]
pub use rustix_be::FileBackend;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::FileBackend;

#[cfg(not(any(windows, unix, target_os = "wasi")))]
mod fallback;
#[cfg(not(any(windows, unix, target_os = "wasi")))]
pub use fallback::FileBackend;
