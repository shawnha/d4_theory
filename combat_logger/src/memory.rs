use std::io::BufRead;

#[derive(Debug)]
pub enum Error {
    /// Process was not found
    ProcessNotFound(String),

    /// Failed to read memory
    ReadMemoryFailed(usize),

    /// Read memory but was incomplete
    ReadMemoryPartial(usize, usize),

    /// Failed to write memory
    WriteMemoryFailed(usize),

    /// Wrote memory but was incomplete
    WriteMemoryPartial(usize, usize),

    /// IO error
    IOError(std::io::Error),

    /// UTF8 conversion error
    UTF8Conversion(std::str::Utf8Error),

    /// Parse int error
    ParseInt(std::num::ParseIntError),

    /// Parse str error
    ParseStr(String),
}

/// Implement the formatter for our custom error type
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ProcessNotFound(e) =>
                write!(f, "Process '{}' not found", e),
            Error::ReadMemoryFailed(addr) =>
                write!(f, "Failed to read memory from address 0x{:x}", addr),
            Error::ReadMemoryPartial(addr, bytes) =>
                write!(f, 
                    "Partial read: only read {} bytes from address 0x{:x}",
                    bytes, addr),
            Error::WriteMemoryFailed(addr) =>
                write!(f, "Failed to write memory at address 0x{:x}", addr),
            Error::WriteMemoryPartial(addr, bytes) =>
                write!(f,
                    "Partial write: only wrote {} bytes at address 0x{:x}",
                    bytes, addr),
            Error::IOError(e) =>
                write!(f, "IO error: {}", e),
            Error::UTF8Conversion(e) =>
                write!(f, "UTF8 conversion error: {}", e),
            Error::ParseInt(e) =>
                write!(f, "Parse int error: {}", e),
            Error::ParseStr(e) =>
                write!(f, "Parse str error: {}", e),
        }
    }
}

/// Implement standard error trait and conversion from other error types
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::UTF8Conversion(err)
    }
}
impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseInt(err)
    }
}
impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::ParseStr(err.to_string())
    }
}

/// Custom Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Custom memory range type
pub type MemoryRange = core::ops::Range<u64>;

const CHUNK_SIZE: usize = 256;

pub struct MemoryReader {
    /// Process identifier
    pub process_id: i32,
}

impl MemoryReader {
    /// Create a new memory reader for given process name
    pub fn new(process_name: &str) -> Result<Self> {
        Ok(Self {
            process_id: Self::find_process(process_name)?,
        })
    }

    /// Find a process by name
    pub fn find_process(name: &str) -> Result<i32> {
        // Place quotes around the process name to handle any spaces
        let name = format!("\"{}\"", name);

        // Execute `ps` and `grep` commands through a shell with root access
        let command = format!("sudo ps -e | grep -w {}", name);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let reader = std::io::BufReader::new(output.stdout
            .ok_or("Failed to open stdout")?);

        for line in reader.lines() {
            // Unwrap the result to get the line
            let line = line?;

            // Split the line into whitespace-seperated parts
            let parts: Vec<&str> = line.split_whitespace().collect();

            // The first element should be the process id
            if !parts.is_empty() {
                let process_id = parts[0].parse::<i32>()?;
                return Ok(process_id);
            }
        }

        Err(Error::ProcessNotFound(name))
    }

    /// Reads bytes from a process at the given address for the given length
    pub fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>> {
        // Setup local/remote IO vectors for our buffer and memory that we 
        // are reading
        let mut buffer = vec![0u8; len];
        let local_iovec = libc::iovec {
            iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
            iov_len: len,
        };
        let remote_iovec = libc::iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: len,
        };

        // Perform the read operation using PROCESS_VM_READV syscall
        let bytes_read = unsafe {
            libc::process_vm_readv(
                self.process_id as libc::pid_t,
                &local_iovec as *const libc::iovec,
                1,
                &remote_iovec as *const libc::iovec,
                1,
                0,
            )
        };

        // Check the result of the read operation
        if bytes_read == -1 {
            Err(Error::ReadMemoryFailed(address))
        }
        else if bytes_read != len as isize {
            Err(Error::ReadMemoryPartial(address, bytes_read as usize))
        }
        else {
            Ok(buffer)
        }
    }

    /// Reads a string from a process at the given address for the given range
    pub fn read_string(&self, range: MemoryRange) -> Result<String> {
        let mut buffer = vec![];
        let mut start = range.start as usize;
        let mut reached_end = false;

        while start < range.end as usize {
            // Calculate the length to read, making sure not to overflow
            let mut length_to_read = CHUNK_SIZE;
            if start + length_to_read > range.end as usize {
                length_to_read = range.end as usize - start;
            }

            // Read a chunk of memory
            let chunk_result = self.read_bytes(start, length_to_read);
            match chunk_result {
                Ok(chunk) => {
                    // Check if there is a null terminator in the chunk
                    if let Some(end) = chunk.iter().position(|&b| b == 0) {
                        // Null terminator found
                        buffer.extend_from_slice(&chunk[..end]);
                        break;
                    }
                    else if chunk.len() < length_to_read {
                        // Less data was read than requested
                        buffer.extend(chunk);
                        reached_end = true;
                        break;
                    }
                    else {
                        // Proceed normally
                        buffer.extend(chunk);
                    }

                    // Move to the next block of memory
                    start += length_to_read;
                }
                Err(e) => { return Err(e); }
            }
        }

        // Convert buffer to a String
        String::from_utf8(buffer)
            .map_err(|_| Error::ReadMemoryFailed(range.start as usize))
            .and_then(|s| {
                if reached_end {
                    Err(
                        Error::ReadMemoryPartial(range.start as usize, s.len())
                    )
                }
                else {
                    Ok(s)
                }
            })
    }

    /// Writes bytes to a process at the given address
    pub fn write_bytes(&self, address: usize, data: &[u8]) -> Result<()> {
        // Setup local/remote IO vectors for our buffer and memory that we 
        // are writing to
        let local_iovec = libc::iovec {
            iov_base: data.as_ptr() as *mut libc::c_void,
            iov_len: data.len(),
        };
        let remote_iovec = libc::iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: data.len(),
        };

        // Perform the write operation using PROCESS_VM_WRITEV syscall
        let bytes_written = unsafe {
            libc::process_vm_writev(
                self.process_id as libc::pid_t,
                &local_iovec as *const libc::iovec,
                1,
                &remote_iovec as *const libc::iovec,
                1,
                0,
            )
        };

        // Check the result of the write operation
        if bytes_written == -1 {
            Err(Error::WriteMemoryFailed(address))
        }
        else if bytes_written != data.len() as isize {
            Err(Error::WriteMemoryPartial(address, bytes_written as usize))
        }
        else {
            Ok(())
        }
    }
}
