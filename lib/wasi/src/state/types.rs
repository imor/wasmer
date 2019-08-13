/// types for use in the WASI filesystem
use crate::syscalls::types::*;
use std::{
    fs,
    io::{self, Read, Seek, Write},
    path::PathBuf,
    time::SystemTime,
};

/// Error type for external users
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
// dead code beacuse this is for external use
pub enum WasiFsError {
    /// The fd given as a base was not a directory so the operation was not possible
    BaseNotDirectory,
    /// Expected a file but found not a file
    NotAFile,
    /// The fd given was not usable
    InvalidFd,
    /// File exists
    AlreadyExists,
    /// Something failed when doing IO. These errors can generally not be handled.
    /// It may work if tried again.
    IOError,
    /// The address was in use
    AddressInUse,
    /// The address could not be found
    AddressNotAvailable,
    /// A pipe was closed
    BrokenPipe,
    /// The connection was aborted
    ConnectionAborted,
    /// The connection request was refused
    ConnectionRefused,
    /// The connection was reset
    ConnectionReset,
    /// The operation was interrupted before it could finish
    Interrupted,
    /// Invalid internal data, if the argument data is invalid, use `InvalidInput`
    InvalidData,
    /// The provided data is invalid
    InvalidInput,
    /// Could not perform the operation because there was not an open connection
    NotConnected,
    /// The requested file or directory could not be found
    EntityNotFound,
    /// Caller was not allowed to perform this operation
    PermissionDenied,
    /// The operation did not complete within the given amount of time
    TimedOut,
    /// Found EOF when EOF was not expected
    UnexpectedEof,
    /// Operation would block, this error lets the caller know that they can try again
    WouldBlock,
    /// A call to write returned 0
    WriteZero,
    /// A WASI error without an external name.  If you encounter this it means
    /// that there's probably a bug on our side (maybe as simple as forgetting to wrap
    /// this error, but perhaps something broke)
    UnknownError(__wasi_errno_t),
}

impl WasiFsError {
    pub fn from_wasi_err(err: __wasi_errno_t) -> WasiFsError {
        match err {
            __WASI_EBADF => WasiFsError::InvalidFd,
            __WASI_EEXIST => WasiFsError::AlreadyExists,
            __WASI_EIO => WasiFsError::IOError,
            __WASI_EADDRINUSE => WasiFsError::AddressInUse,
            __WASI_EADDRNOTAVAIL => WasiFsError::AddressNotAvailable,
            __WASI_EPIPE => WasiFsError::BrokenPipe,
            __WASI_ECONNABORTED => WasiFsError::ConnectionAborted,
            __WASI_ECONNREFUSED => WasiFsError::ConnectionRefused,
            __WASI_ECONNRESET => WasiFsError::ConnectionReset,
            __WASI_EINTR => WasiFsError::Interrupted,
            __WASI_EINVAL => WasiFsError::InvalidInput,
            __WASI_ENOTCONN => WasiFsError::NotConnected,
            __WASI_ENOENT => WasiFsError::EntityNotFound,
            __WASI_EPERM => WasiFsError::PermissionDenied,
            __WASI_ETIMEDOUT => WasiFsError::TimedOut,
            __WASI_EPROTO => WasiFsError::UnexpectedEof,
            __WASI_EAGAIN => WasiFsError::WouldBlock,
            __WASI_ENOSPC => WasiFsError::WriteZero,
            _ => WasiFsError::UnknownError(err),
        }
    }

    pub fn into_wasi_err(self) -> __wasi_errno_t {
        match self {
            WasiFsError::AlreadyExists => __WASI_EEXIST,
            WasiFsError::AddressInUse => __WASI_EADDRINUSE,
            WasiFsError::AddressNotAvailable => __WASI_EADDRNOTAVAIL,
            WasiFsError::BaseNotDirectory => __WASI_ENOTDIR,
            WasiFsError::BrokenPipe => __WASI_EPIPE,
            WasiFsError::ConnectionAborted => __WASI_ECONNABORTED,
            WasiFsError::ConnectionRefused => __WASI_ECONNREFUSED,
            WasiFsError::ConnectionReset => __WASI_ECONNRESET,
            WasiFsError::Interrupted => __WASI_EINTR,
            WasiFsError::InvalidData => __WASI_EIO,
            WasiFsError::InvalidFd => __WASI_EBADF,
            WasiFsError::InvalidInput => __WASI_EINVAL,
            WasiFsError::IOError => __WASI_EIO,
            WasiFsError::NotAFile => __WASI_EINVAL,
            WasiFsError::NotConnected => __WASI_ENOTCONN,
            WasiFsError::EntityNotFound => __WASI_ENOENT,
            WasiFsError::PermissionDenied => __WASI_EPERM,
            WasiFsError::TimedOut => __WASI_ETIMEDOUT,
            WasiFsError::UnexpectedEof => __WASI_EPROTO,
            WasiFsError::WouldBlock => __WASI_EAGAIN,
            WasiFsError::WriteZero => __WASI_ENOSPC,
            WasiFsError::UnknownError(ec) => ec,
        }
    }
}

/// This trait relies on your file closing when it goes out of scope via `Drop`
pub trait WasiFile: std::fmt::Debug + Write + Read + Seek {
    /// the last time the file was accessed in nanoseconds as a UNIX timestamp
    fn last_accessed(&self) -> __wasi_timestamp_t;
    /// the last time the file was modified in nanoseconds as a UNIX timestamp
    fn last_modified(&self) -> __wasi_timestamp_t;
    /// the time at which the file was created in nanoseconds as a UNIX timestamp
    fn created_time(&self) -> __wasi_timestamp_t;
    /// set the last time the file was accessed in nanoseconds as a UNIX timestamp
    // TODO: stablize this in 0.7.0 by removing default impl
    fn set_last_accessed(&self, _last_accessed: __wasi_timestamp_t) {
        panic!("Default implementation for compatibilty in the 0.6.X releases; this will be removed in 0.7.0.  Please implement WasiFile::set_last_accessed for your type before then");
    }
    /// set the last time the file was modified in nanoseconds as a UNIX timestamp
    // TODO: stablize this in 0.7.0 by removing default impl
    fn set_last_modified(&self, _last_modified: __wasi_timestamp_t) {
        panic!("Default implementation for compatibilty in the 0.6.X releases; this will be removed in 0.7.0.  Please implement WasiFile::set_last_modified for your type before then");
    }
    /// set the time at which the file was created in nanoseconds as a UNIX timestamp
    // TODO: stablize this in 0.7.0 by removing default impl
    fn set_created_time(&self, _created_time: __wasi_timestamp_t) {
        panic!("Default implementation for compatibilty in the 0.6.X releases; this will be removed in 0.7.0.  Please implement WasiFile::set_created_time for your type before then");
    }
    /// the size of the file in bytes
    fn size(&self) -> u64;
    /// Change the size of the file, if the `new_size` is greater than the current size
    /// the extra bytes will be allocated and zeroed
    // TODO: stablize this in 0.7.0 by removing default impl
    fn set_len(&mut self, _new_size: __wasi_filesize_t) -> Result<(), WasiFsError> {
        panic!("Default implementation for compatibilty in the 0.6.X releases; this will be removed in 0.7.0.  Please implement WasiFile::allocate for your type before then");
    }

    /// Request deletion of the file
    // TODO: break this out into a WasiPath trait which is dynamically in Kind::File
    // this change can't be done until before release
    fn unlink(&mut self) -> Result<(), WasiFsError> {
        panic!("Default implementation for compatibilty in the 0.6.X releases; this will be removed in 0.7.0.  Please implement WasiFile::unlink for your type before then");
    }

    /// Store file contents and metadata to disk
    // TODO: stablize this in 0.7.0 by removing default impl
    fn sync_to_disk(&self) -> Result<(), WasiFsError> {
        panic!("Default implementation for compatibilty in the 0.6.X releases; this will be removed in 0.7.0.  Please implement WasiFile::sync_to_disk for your type before then");
    }

    /// Moves the file to a new location
    /// NOTE: the signature of this function will change before stabilization
    // TODO: stablizie this in 0.7.0 or 0.8.0 by removing default impl
    fn rename_file(&self, _new_name: &std::path::Path) -> Result<(), WasiFsError> {
        panic!("Default implementation for compatibilty in the 0.6.X releases; this will be removed in 0.7.0 or 0.8.0.  Please implement WasiFile::rename_file for your type before then");
    }
}

pub trait WasiPath {}

/// A thin wrapper around `std::fs::File`
#[derive(Debug)]
pub struct HostFile {
    pub inner: fs::File,
    pub host_path: PathBuf,
}

impl HostFile {
    /// creates a new host file from a `std::fs::File` and a path
    pub fn new(file: fs::File, host_path: PathBuf) -> Self {
        Self {
            inner: file,
            host_path,
        }
    }

    pub fn metadata(&self) -> fs::Metadata {
        self.inner.metadata().unwrap()
    }
}

impl Read for HostFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }
}
impl Seek for HostFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}
impl Write for HostFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }
    fn write_fmt(&mut self, fmt: ::std::fmt::Arguments) -> io::Result<()> {
        self.inner.write_fmt(fmt)
    }
}

impl WasiFile for HostFile {
    fn last_accessed(&self) -> u64 {
        self.metadata()
            .accessed()
            .ok()
            .and_then(|ct| ct.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|ct| ct.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn set_last_accessed(&self, _last_accessed: __wasi_timestamp_t) {
        // TODO: figure out how to do this
    }

    fn last_modified(&self) -> u64 {
        self.metadata()
            .modified()
            .ok()
            .and_then(|ct| ct.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|ct| ct.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn set_last_modified(&self, _last_modified: __wasi_timestamp_t) {
        // TODO: figure out how to do this
    }

    fn created_time(&self) -> u64 {
        self.metadata()
            .created()
            .ok()
            .and_then(|ct| ct.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|ct| ct.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn set_created_time(&self, _created_time: __wasi_timestamp_t) {
        // TODO: figure out how to do this
    }

    fn size(&self) -> u64 {
        self.metadata().len()
    }

    fn set_len(&mut self, new_size: __wasi_filesize_t) -> Result<(), WasiFsError> {
        fs::File::set_len(&self.inner, new_size).map_err(Into::into)
    }

    fn unlink(&mut self) -> Result<(), WasiFsError> {
        std::fs::remove_file(&self.host_path).map_err(Into::into)
    }
    fn sync_to_disk(&self) -> Result<(), WasiFsError> {
        self.inner.sync_all().map_err(Into::into)
    }

    fn rename_file(&self, new_name: &std::path::Path) -> Result<(), WasiFsError> {
        std::fs::rename(&self.host_path, new_name).map_err(Into::into)
    }
}

impl From<io::Error> for WasiFsError {
    fn from(io_error: io::Error) -> Self {
        match io_error.kind() {
            io::ErrorKind::AddrInUse => WasiFsError::AddressInUse,
            io::ErrorKind::AddrNotAvailable => WasiFsError::AddressNotAvailable,
            io::ErrorKind::AlreadyExists => WasiFsError::AlreadyExists,
            io::ErrorKind::BrokenPipe => WasiFsError::BrokenPipe,
            io::ErrorKind::ConnectionAborted => WasiFsError::ConnectionAborted,
            io::ErrorKind::ConnectionRefused => WasiFsError::ConnectionRefused,
            io::ErrorKind::ConnectionReset => WasiFsError::ConnectionReset,
            io::ErrorKind::Interrupted => WasiFsError::Interrupted,
            io::ErrorKind::InvalidData => WasiFsError::InvalidData,
            io::ErrorKind::InvalidInput => WasiFsError::InvalidInput,
            io::ErrorKind::NotConnected => WasiFsError::NotConnected,
            io::ErrorKind::NotFound => WasiFsError::EntityNotFound,
            io::ErrorKind::PermissionDenied => WasiFsError::PermissionDenied,
            io::ErrorKind::TimedOut => WasiFsError::TimedOut,
            io::ErrorKind::UnexpectedEof => WasiFsError::UnexpectedEof,
            io::ErrorKind::WouldBlock => WasiFsError::WouldBlock,
            io::ErrorKind::WriteZero => WasiFsError::WriteZero,
            io::ErrorKind::Other => WasiFsError::IOError,
            // if the following triggers, a new error type was added to this non-exhaustive enum
            _ => WasiFsError::UnknownError(__WASI_EIO),
        }
    }
}

#[derive(Debug)]
pub struct Stdout(pub std::io::Stdout);
impl Read for Stdout {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stdout",
        ))
    }
    fn read_to_end(&mut self, _buf: &mut Vec<u8>) -> io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stdout",
        ))
    }
    fn read_to_string(&mut self, _buf: &mut String) -> io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stdout",
        ))
    }
    fn read_exact(&mut self, _buf: &mut [u8]) -> io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stdout",
        ))
    }
}
impl Seek for Stdout {
    fn seek(&mut self, _pos: io::SeekFrom) -> io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not seek stdout",
        ))
    }
}
impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }
    fn write_fmt(&mut self, fmt: ::std::fmt::Arguments) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }
}

impl WasiFile for Stdout {
    fn last_accessed(&self) -> u64 {
        0
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn created_time(&self) -> u64 {
        0
    }
    fn size(&self) -> u64 {
        0
    }
}

#[derive(Debug)]
pub struct Stderr(pub std::io::Stderr);
impl Read for Stderr {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stderr",
        ))
    }
    fn read_to_end(&mut self, _buf: &mut Vec<u8>) -> io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stderr",
        ))
    }
    fn read_to_string(&mut self, _buf: &mut String) -> io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stderr",
        ))
    }
    fn read_exact(&mut self, _buf: &mut [u8]) -> io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stderr",
        ))
    }
}
impl Seek for Stderr {
    fn seek(&mut self, _pos: io::SeekFrom) -> io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not seek stderr",
        ))
    }
}
impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }
    fn write_fmt(&mut self, fmt: ::std::fmt::Arguments) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }
}

impl WasiFile for Stderr {
    fn last_accessed(&self) -> u64 {
        0
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn created_time(&self) -> u64 {
        0
    }
    fn size(&self) -> u64 {
        0
    }
}

#[derive(Debug)]
pub struct Stdin(pub std::io::Stdin);
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.0.read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.0.read_exact(buf)
    }
}
impl Seek for Stdin {
    fn seek(&mut self, _pos: io::SeekFrom) -> io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not seek stdin",
        ))
    }
}
impl Write for Stdin {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not write to stdin",
        ))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not write to stdin",
        ))
    }
    fn write_all(&mut self, _buf: &[u8]) -> io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not write to stdin",
        ))
    }
    fn write_fmt(&mut self, _fmt: ::std::fmt::Arguments) -> io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not write to stdin",
        ))
    }
}

impl WasiFile for Stdin {
    fn last_accessed(&self) -> u64 {
        0
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn created_time(&self) -> u64 {
        0
    }
    fn size(&self) -> u64 {
        0
    }
}

/*
TODO: Think about using this
trait WasiFdBacking: std::fmt::Debug {
    fn get_stat(&self) -> &__wasi_filestat_t;
    fn get_stat_mut(&mut self) -> &mut __wasi_filestat_t;
    fn is_preopened(&self) -> bool;
    fn get_name(&self) -> &str;
}
*/