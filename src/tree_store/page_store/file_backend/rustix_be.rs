use crate::{DatabaseError, Result, StorageBackend};
use std::fs::File;
use std::io;

use rustix::{fd::AsFd, fs::FileExt};

/// Stores a database as a file on-disk.
#[derive(Debug)]
pub struct FileBackend {
    file: File,
}

impl FileBackend {
    pub fn new(file: File) -> Result<Self, DatabaseError> {
        let fd = file.as_fd();
        match rustix::fs::flock(fd, rustix::fs::FlockOperation::LockExclusive) {
            Ok(_) => Ok(Self { file }),
            Err(err) => {
                if err.kind() == io::ErrorKind::WouldBlock {
                    Err(DatabaseError::DatabaseAlreadyOpen)
                } else {
                    Err(io::Error::from(err).into())
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

    fn sync_data(&self, _: bool) -> Result<(), io::Error> {
        self.file.sync_data()
    }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), io::Error> {
        self.file.write_all_at(data, offset)
    }
}

impl Drop for FileBackend {
    fn drop(&mut self) {
        let fd = self.file.as_fd();
        rustix::fs::flock(fd, rustix::fs::FlockOperation::Unlock).unwrap();
    }
}
