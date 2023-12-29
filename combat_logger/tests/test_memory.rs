use std::io::BufRead;
use combat_logger::memory::MemoryReader;

fn run_test_binary() -> (i32, usize, usize) {
    let project_dir = std::env::current_dir()
        .expect("Failed to get current directory");
    let source_path = project_dir.join("tests").join("test_binary.rs");
    let binary_path = project_dir.join("tests").join("test_binary");
    let compile = std::process::Command::new("rustc")
        .args(&[source_path, "-o".into(), binary_path.clone()])
        .status()
        .expect("Failed to compile the test binary");

    let mut child = std::process::Command::new(binary_path)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to run the test binary");
    let process_id = child.id() as i32;

    let output = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut memory_range = (0, 0);
    for line in output.lines() {
        let line = line.expect("Failed to read line from binary stdout");
        let parts: Vec<&str> = line.split('-')
            .map(|s| s.trim_start_matches("0x"))
            .collect();
        if parts.len() == 2 {
            if let Ok(start) = usize::from_str_radix(parts[0], 16) {
                if let Ok(end) = usize::from_str_radix(parts[1], 16) {
                    memory_range = (start, end);
                    break;
                }
            }
        }
    }

    if memory_range.0 == 0 && memory_range.1 == 0 {
        panic!("Failed to parse memory range from test binary stdout");
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    (process_id, memory_range.0, memory_range.1 - memory_range.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_bytes() {
        let (process_id, start_addr, size) = run_test_binary();
        std::thread::sleep(std::time::Duration::from_secs(1));

        let test_data_size = size / 2;
        let test_data: Vec<u8> = vec![0xAB; test_data_size];

        let memory_reader = MemoryReader {
            process_id: process_id,
        };

        let write_result = memory_reader.write_bytes(start_addr, &test_data);
        assert!(write_result.is_ok(), 
            "Failed to write to memory region {:?}",
            write_result
        );

        let read_result = memory_reader.read_bytes(start_addr, test_data_size);
        assert!(read_result.is_ok(),
            "Failed to read from memory region {:?}",
            read_result
        );

        let read_data = read_result.unwrap();
        assert_eq!(test_data, read_data, "Read data does not match test data");

        let project_dir = std::env::current_dir()
            .expect("Failed to get current directory");
        let binary_path = project_dir.join("tests").join("test_binary");
        std::process::Command::new("kill")
            .arg(process_id.to_string())
            .status()
            .expect("Failed to kill the test process");
        std::fs::remove_file(binary_path)
            .expect("Failed to remove test binary");
    }
}
