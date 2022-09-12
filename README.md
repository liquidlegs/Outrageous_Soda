# Outrageous SODA

Outrageous SODA is a simple web fuzzer that can guess URL directory names and fuzz URL parameters.
This project is pretty bare-bones at the moment and currently offers the following features.

- Generate URI paths from a wordlist
- Make get requests and display successful responses
- Display all debugging information (Error messages and success/failed HTTP(s) responses)
- Display HTML responses
- Control the timeout in milliseconds between each response
- Multithreading (Not customizable just yet)
- Parameter fuzzing (Not implemented)
- Write all output to a file (Not implemented)
- Mostly polished CLI interface