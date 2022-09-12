use clap::Parser;
use std::io::{Write, BufReader, BufRead, Error as IoError, ErrorKind};
use std::fs::OpenOptions;
use std::thread;
use core::time::Duration;
use reqwest;
use reqwest::Error;
use std::borrow::Cow;
use reqwest::blocking::Response;

pub const LARGE_FILE: usize = 1000000;
pub static F_HTTP: &str = "http://";
pub static F_HTTPS: &str = "https://";

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct SodaArgs {
  /// Url
  #[clap(value_parser, value_name = "URL", help = "The base url in the GET request")]
  pub url: String,

  /// WordList
  #[clap(value_parser, value_name = "FILE", help = "A wordlist used for generating GET requests")]
  pub wordlist: String,

  /// Fuzz
  #[clap(value_enum, help = "Fuzz a URI path or paramater")]
  pub fuzz: Fuzz,

  /// Debug
  #[clap(long, value_name = "no_value", default_value_if("debug", Some("false"), Some("true")), min_values(0), help = "Shows error messages and all server responses")]
  pub debug: bool,

  /// Debug Detail
  #[clap(long, value_name = "no_value", default_value_if("verbose", Some("false"), Some("true")), min_values(0), help = "Shows all html responses")]
  pub verbose: bool,

  /// Output file
  #[clap(short, long, value_name = "FILE", value_parser, help = "Output results to a file")]
  pub output: Option<String>,

  /// Timeout (miliseconds)
  #[clap(short, long, default_value = "300", value_name = "INT", help = "The timeout period before the connection is dropped in miliseconds")]
  pub timeout: u64,

  /// Threads
  #[clap(short = 'T', long, default_value = "0", value_name = "INT", help = "The number of threads you wish to use to process requests")]
  pub threads: u32,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum Fuzz {
  DirectoryPath,
  Parameter
}

impl SodaArgs {
  /**Function prints the contents of the setting structure to the screen. */
  pub fn show_information(&self) -> () {
    println!("\nurl=[{:?}]\nwordlist=[{:?}]\noutput=[{:?}]\ndebug=[{:?}]\nverbose=[{:?}]\ntimeout=[{:?}]",
      self.url, self.wordlist, self.output, self.debug, self.verbose, self.timeout
    );
  }

  /**Function writes the contetns of the buffer to the disk.
   * Params:
   *  &self
   *  buffer: &str {The content to be written to the disk.}
   * Returns bool.
   */
  #[allow(unused_assignments)]
  pub fn write_file_to_disk(&self, buffer: &str) -> Result<usize, IoError> {
    let mut output_name = "".to_owned();
    let out_name = self.output.clone();
    
    match out_name {
      Some(out) => { output_name = out; }
      None => {
        let e = IoError::new(ErrorKind::NotFound, "output_file_name supplied was None");
        return Err(e);
      }
    }

    let write_file = OpenOptions::new().read(true).write(true).open(output_name.as_str());

    match write_file {
      Ok(mut s) => {
        match s.write(buffer.as_bytes()) {
          Ok(f) => { return Ok(f); }
          Err(e) => { return Err(e); }
        }
      },

      Err(e) => { return Err(e); }
    }
  }

  /**Function checks if a file exists on disk.
   * Params:
   *  file_name: &str {The name of the file}
   * Returns bool
   */
  pub fn file_exists(file_name: &str) -> bool {
    match OpenOptions::new().read(true).open(file_name) {
      Ok(_) => { return true; },
      Err(_) => { return false; }
    }
  }

  /**Function creates an empty text file
   * Params:
   *  file:name: &str {The name of the file}
   * Returns Result<bool, Error>
  */
  pub fn create_file(file_name: &str) -> Result<bool, IoError> {
    match OpenOptions::new().read(true).write(true).create(true).open(file_name) {
      Ok(_) => { return Ok(true); }
      Err(e) => { return Err(e); }
    }
  }

  /**Function sends a simple get request and displays the server repsonse to the screen.
 * Params:
 *  &self
 *  split_wordlist: &str {A chunk of the input wordlist that be handed off to a thread.}
 * Returns JoinHandle<()>
 */
  pub fn thread_get_request(&self, split_wordlist: String) -> thread::JoinHandle<()> {
    let debug = self.debug.clone();
    let verbose = self.verbose.clone();
    let timeout = self.timeout.clone();
    
    // Create the thread.
    let handle = thread::spawn(move || {
      let slices: Vec<&str> = split_wordlist.split(" ").collect();        // Create array of slices.

      let mut request = "".to_owned();                            // Builds the request
      for i in slices {
        request.push_str(i);

        match Self::get(request.as_str(), timeout) {    // Sends the GET reuqest.
          Ok(s) => {
            let status = s.status();
            if status.is_success() && debug == false {
              println!("{} -- {}", request, status);                     // print OK 200 for successful connections.
            }

            if debug == true {                                           // Enable debugging to print everything.
              println!("{} -- {}", request, status);
              
              if verbose == true {                                     // Enable this flag to get the html body.
                match s.text() {
                  Ok(body) => {
                    println!("|\n|\n{}", body);
                  }
                  Err(e) => { println!("{}", e); }
                }
              }
            }
          },
          Err(e) => {
            if debug == true {
              println!("\n{}\n__________________________________________________", e);
            }
            
            else if e.is_builder() != true {
              println!("\n{}\n__________________________________________________", e);
            }
          }
        }

        request.clear();
      }
    });

    if debug == true {
      println!("starting thread {:?}", handle.thread().id());
    }

    handle
  }

  pub fn standard_get_request(&self, split_wordlist: String) -> () {
    let debug = self.debug.clone();
    let verbose = self.verbose.clone();
    let timeout = self.timeout.clone();
    
    let slices: Vec<&str> = split_wordlist.split(" ").collect();        // Create array of slices.
    let mut request = "".to_owned();                            // Builds the request
    for i in slices {
      request.push_str(i);

      match Self::get(request.as_str(), timeout) {                 // Sends the GET reuqest.
        Ok(s) => {
          let status = s.status();
          if status.is_success() && debug == false {
            println!("{} -- {}", request, status);                     // print OK 200 for successful connections.
          }

          if debug == true {                                           // Enable debugging to print everything.
            println!("{} -- {}", request, status);
            
            if verbose == true {                                      // Enable this flag to get the html body.
              match s.text() {
                Ok(body) => {
                  println!("|\n|\n{}", body);
                }
                Err(e) => { println!("{}", e); }
              }
            }
          }
        },
        Err(e) => {
          if debug == true {
            println!("\n{}\n__________________________________________________", e);
          }
          
          else if e.is_builder() != true {
            println!("\n{}\n__________________________________________________", e);
          }
        }
      }

      request.clear();
    }
  }

    /**Function sends a get request to a web server and returns the response.
   * Params:
   *  url:      &str     {The web address to make the request.}
   *  timeout:  u64      {The time in miliseconds before the request is dropped.}
   *  dbg:      bool     {Displays the status code and the html response to the screen.}
   * Returns StatusCode.
   */
  pub fn get(url: &str, timeout: u64) -> Result<Response, Error> {
    let builder = reqwest::blocking::ClientBuilder::new();
    let timeout = builder.timeout(Duration::from_millis(timeout));

    match timeout.build() {
      Ok(client) => {
        match client.get(url).send() {
          Ok(s) => { return Ok(s); }
          Err(e) => { return Err(e); }
        }  
      },

      Err(e) => { return Err(e); }
    }
  }

    /**Function reads file into a buffer and returns it as a string and the total number of bytres that were read.
   * Params:
   *  wlist: &str {The file path and name to the wordlist.}
   * Returns (String, usize)
   */
  pub fn parse_wordlist(&self) -> (String, usize) {
    let mut byte_array = vec![];                                               // Creates vector to stores bytes.
    let mut total_bytes_read: usize = 0;

    match OpenOptions::new().read(true).open(self.wordlist.as_str()) {
      Ok(read_file) => {
        let mut reader = BufReader::new(read_file);

        match reader.read_until(u8::MIN, &mut byte_array) {                      // Reads the entire file and stores in a buffer.
          Ok(b) => { total_bytes_read += b; },
          Err(e) => {
            if self.debug == true { println!("{}", e.kind()); }
          }
        }
      },

      Err(e) => {
        if self.debug == true { println!("{}", e.kind()); }
      }
    }
    
    let string_ptr = String::from_utf8_lossy(&byte_array);                   // Creates a string and removed on utf8 chars.
    let mut utf8_string = String::new();
    match string_ptr {
      Cow::Borrowed(b) => { utf8_string.push_str(b); },
      Cow::Owned(b) => { utf8_string.push_str(b.as_str()); }
    }

    (utf8_string, total_bytes_read)
  }

    /**Function makes get requests depending on the base url and the contents of the supplied wordlist.
   * Params:
   *  set: Settings {The settings or command line arguments that the user supplied.}
   * Returns nothing.
   */
  pub fn fuzz_directory(&self) -> () {
    let file_contents = self.parse_wordlist();                      // Gets the contents of the wordlist
    if file_contents.1 >= LARGE_FILE {
      println!("Wanring: word list is larger than 50MB. Performance may be slow...");
    }
    
    // Array is split into slice elements 
    let slice_array: Vec<&str> = file_contents.0.split("\r\n").collect();
    let mut temp_string = "".to_owned();                                         // String holds elements to be processed.
    let mut chunk_counter: usize = 0;                                                    // Counts the number of elements.
    let mut handles = vec![];                                       // Stores the thread handles.
    
    let mut url = "".to_owned();
    url.push_str(self.url.as_str());

    let sl_array_len = slice_array.len().clone();

    // Allocates memory to string that holds 20 or less elements
    for chunk in slice_array {
      if chunk_counter >= 20 {
        // The string is cloned and passed to the thread.
        let mut c_temp_string = String::new();
        c_temp_string.push_str(temp_string.as_str());

        let test_handle = self.thread_get_request(c_temp_string);
        handles.push(test_handle);
        temp_string.clear();
        chunk_counter = 0;
      }
      
      // Setup each element and push them to the string.
      temp_string.push_str(url.as_str());
      temp_string.push_str(chunk);
      
      if chunk_counter < sl_array_len {
        temp_string.push(' ');
      }

      chunk_counter += 1;
    }
    
    // Run thread for left over elements that did not exceed past 20.
    if temp_string.len() > 0 {
      let mut last_string = String::new();
      last_string.push_str(temp_string.as_str());
      self.standard_get_request(last_string);
    }

    // Sleep for 2 seconds and join threads into the main thread.
    thread::sleep(Duration::from_millis(1000));
    for i in handles {
      if i.is_finished() == true {
        i.join().unwrap();
      }
    }
  }
}