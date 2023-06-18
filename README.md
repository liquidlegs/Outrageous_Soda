# Outrageous Soda

Outrageous SODA is a simple web fuzzer that can guess URL directory names and fuzz URL parameters.
The project currently offers the following features.

- Generate directory paths from a wordlist
- Generate requests from a list of file extensions
- Display debug information, HTML responses and status codes
- Control the timeout in milliseconds between each response
- Multithreading
- Parameter fuzzing
- Write all output to a file

# Compilation Instructions
1) Download and install rustup here if not already https://www.rust-lang.org/
2) Add the cargo to your path
3) cd into the project directory
4) Run the following command and navigate to /target/build/
> Cargo build --release
