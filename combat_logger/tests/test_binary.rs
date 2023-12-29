#![feature(rustc_private)]
extern crate libc;
use libc::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE, MAP_FAILED};
use std::io::Write;

const MEMORY_SIZE: usize = 4096;

fn main() {
    // Allocate a region of memory using `mmap`
    let addr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            MEMORY_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    // Check for `mmap` failures
    if addr == MAP_FAILED {
        eprintln!("mmap failed to allocate memory");
        std::process::exit(1);
    }

    // Print the memory address range to stdout
    println!("{:p}-{:p}", addr, unsafe { addr.add(MEMORY_SIZE) });
    std::io::stdout().flush().expect("Failed to flush stdout");

    // Allow the test harness to recieve the output
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Deallocate the memory region before exiting
    unsafe {
        libc::munmap(addr, MEMORY_SIZE);
    }
}
