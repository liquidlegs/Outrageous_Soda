# Outrageous SODA

Outrageous SODA is a simple web fuzzer that can guess URL directory names and fuzz URL parameters.
This project is pretty bare-bones at the moment and currently offers the following features.

- Generate directory paths from a wordlist
- Make get requests and display successful responses
- Display all debugging information (Error messages and success/failed HTTP(s) responses)
- Display HTML responses
- Multithreading (Not customizable just yet)
- Control the timeout in milliseconds between each response
- Write all output to a file (Partially implemented)
- A half-functioning CLI interface (Buggy and needs fixing)