// TODO once Rust's libc has flock implemented for WASI, this file needs to be revisited.
// What needs to be changed is commented below.
// See also: https://github.com/WebAssembly/wasi-filesystem/issues/2

// Remove this line once wasi-libc has flock
#![cfg_attr(target_os = "wasi", allow(unused_imports))]

use crate::{DatabaseError, Result, StorageBackend};
use std::fs::File;
use std::io;

use rustix::fd::AsFd;


#[cfg(unix)]
use std::os::unix::{fs::FileExt, io::AsRawFd};

#[cfg(target_os = "wasi")]
use std::os::wasi::{fs::FileExt, io::AsRawFd};

/// Stores a database as a file on-disk.
#[derive(Debug)]
pub struct FileBackend {
    file: File,
}

impl FileBackend {
    /// Creates a new backend which stores data to the given file.
    // This is a no-op until we get flock in wasi-libc.
    // Delete this function when we get flock.
    #[cfg(target_os = "wasi")]
    pub fn new(file: File) -> Result<Self, DatabaseError> {
        Ok(Self { file })
    }

    /// Creates a new backend which stores data to the given file.
    #[cfg(unix)] // remove this line when wasi-libc gets flock
    pub fn new(file: File) -> Result<Self, DatabaseError> {
        let fd = file.as_fd();
        match rustix::fs::flock(fd, rustix::fs::FlockOperation::LockExclusive) {
            Ok(_) => Ok(Self {file}),
            Err(_) => {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock {
                    Err(DatabaseError::DatabaseAlreadyOpen)
                } else {
                    Err(err.into())
                }
            }
        }
    }
}

impl StorageBackend for FileBackend {
    fn len(&self) -> Result<u64, io::Error> {
        Ok(self.file.metadata()?.len())
    }

    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, io::Error> {
        let mut buffer = vec![0; len];
        self.file.read_exact_at(&mut buffer, offset)?;
        Ok(buffer)
    }

    fn set_len(&self, len: u64) -> Result<(), io::Error> {
        self.file.set_len(len)
    }

    // #[cfg(not(target_os = "macos"))]
    fn sync_data(&self, _: bool) -> Result<(), io::Error> {
        self.file.sync_data()
    }

    // #[cfg(target_os = "macos")]
    // fn sync_data(&self, eventual: bool) -> Result<(), io::Error> {
    //     if eventual {

    //         let code = unsafe { libc::fcntl(self.file.as_raw_fd(), libc::F_BARRIERFSYNC) };
    //         if code == -1 {
    //             Err(io::Error::last_os_error())
    //         } else {
    //             Ok(())
    //         }
    //     } else {
    //         self.file.sync_data()
    //     }
    // }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), io::Error> {
        self.file.write_all_at(data, offset)
    }
}

#[cfg(unix)] // remove this line when wasi-libc gets flock
impl Drop for FileBackend {
    fn drop(&mut self) {
        let fd = self.file.as_fd();
        rustix::fs::flock(fd, rustix::fs::FlockOperation::Unlock).unwrap();
        // unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}
