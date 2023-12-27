use std::io::BufRead;

#[derive(Debug)]
pub enum Error {
    /// Process was not found
    ProcessNotFound(String),

    /// Failed to read memory
    ReadMemoryFailed(usize),

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
            Error::ReadMemoryFailed(e) =>
                write!(f, "Failed to read memory at address 0x{:x}", e),
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

struct Memory {
    // @TODO:
    //  - find processes
    //  - find modules 
    //  - find symbols
    //  - read/write/set memory
    //  - allocate/protect memory
    //  - scan memory by pattern/signature
    //  - hook/unhook functions
    //  - assemble/dissassemble code (JIT)
    //  - VMT hooking/unhooking
    //  - load/unload modules
    //  - get page information
    //  - enumerate process threads
    process_id: i32,
}

impl Memory {
    /// Find a process by name
    fn find_process(name: &str) -> Result<Self> {
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
                return Ok(Memory { process_id });
            }
        }

        Err(Error::ProcessNotFound(name))
    }
}

pub struct MemoryReader {
    memory: Memory,
}

impl MemoryReader {
    /// Create a new memory reader for given process name
    pub fn new(process_name: &str) -> Result<Self> {
        Ok(Self {
            memory: Memory::find_process(process_name)?,
        })
    }

    /// Gets the process ID from memory
    pub fn get_process_id(self) -> i32 {
        self.memory.process_id
    }
}
