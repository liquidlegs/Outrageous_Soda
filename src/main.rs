use std::{env, slice};
use std::io::{ErrorKind, Write, BufRead, BufReader};
use std::fs::OpenOptions;
use std::process::exit;
use std::borrow::Cow;
use std::thread;
use reqwest;
use reqwest::StatusCode;
use core::time::{Duration};
use reqwest::blocking::Response;
use reqwest::Error;

static SYNTAX: &str = "
___        _                                            ____   ___  ____    _    
/ _ \\ _   _| |_ _ __ __ _  __ _  ___  ___  _   _ ___    / ___| / _ \\|  _ \\  / \\   
| | | | | | | __| '__/ _` |/ _` |/ _ \\/ _ \\| | | / __|   \\___ \\| | | | | | |/ _ \\  
| |_| | |_| | |_| | | (_| | (_| |  __/ (_) | |_| \\__ \\    ___) | |_| | |_| / ___ \\ 
\\___/ \\__,_|\\__|_|  \\__,_|\\__, |\\___|\\___/ \\__,_|___/___|____/ \\___/|____/_/   \\_\
                          |___/                    |_____|                      

-h --help                 --    {Displays this message} 1

-d --dictonary-wordlist   --    {File containing paths to fuzz.} 2

-p --parameter-wordlist   --    {File containing list of words to fuzz parameters. 2
                                Mark parameters to be fuzzed with {} brackets.}

--debug                   --    {Shows debug info and error messages and server responses.} 1

--debug-dtl               --    {Shows the html response.} 1

-o --output               --    {The file for the results to be written to.} [Partially implemented] 2

-t --timeout              --    {The timeout in miliseconds.} 2

-T --threads              --    {The number of threads} [Not implemented!]

Example:

 program.exe <Base URL> <wordlist flag> <wordlist> <Opt> <Opt> <Opt> <Opt> <Opt> <Opt> <Opt>

 program.exe 127.0.0.1/dashboard/files/exercise/ -d dictonary.txt -o output.txt -D -v -t 50
 program.exe 127.0.0.1/dashboard/files/exercise?username={}&password={} -p params.txt -o output.txt -t 50  ";

const S_HELP: &str = "-h";
const L_HELP: &str = "--help";
const F_DEBUG: &str = "--debug";
const F_DEBUG_DTL: &str = "--debug-dtl";
const S_OUT: &str = "-o";
const L_OUT: &str = "--output";
const S_TIME: &str = "-t";
const L_TIME: &str = "--timeout";
const S_DICT: &str = "-d";
const L_DICT: &str = "--dictonary-wordlist";
const S_PARAM: &str = "-p";
const L_PARAM: &str = "--parameter-wordlist";
const F_HTTP: &str = "http";
const F_HTTPS: &str = "https";
const DF_TIMEOUT: u64 = 100;
const LARGE_FILE: usize = 1000000;

#[derive(Debug, PartialEq)]
pub enum ListType {
  Dictonary,
  Parameter,
}

#[derive(Debug, Copy, Clone)]
pub struct Settings<'a> {
  url: &'a str,
  wlist: &'a str,
  o_file_name: &'a str,
  debug: bool,
  debug_dtl: bool,
  tout: u64,
}

impl <'a>Settings<'a> {

  /**Function creates/initalizes a new Setting structure. */
  pub fn new() -> Settings<'a> {
    Settings {
      url: "",
      wlist: "",
      o_file_name: "",
      debug: false,
      debug_dtl: false,
      tout: 1000,
    }
  }

  /**Function prints the contents of the setting structure to the screen. */
  pub fn show_information(&self) -> () {
    println!("\nurl=[{}]\nwordlist=[{}]\noutput=[{}]\ndebug=[{}]\ndebug_dtl=[{}]\ntimeout=[{}]",
      self.url, self.wlist, self.o_file_name, self.debug, self.debug_dtl, self.tout
    );
  }

  /**Function writes the contetns of the buffer to the disk.
   * Params:
   *  &self
   *  buffer: &str {The content to be written to the disk.}
   * Returns bool.
   */
  pub fn write_file_to_disk(&self, buffer: &str) -> bool {
    let mut out = false;
    let write_file = OpenOptions::new().read(true).write(true).open(self.o_file_name);

    match write_file {
      Ok(mut s) => {
        match s.write(buffer.as_bytes()) {
          Ok(f) => {
            println!("{} bytes has been written to {}", f, self.o_file_name);
            out = true;
          },
          Err(e) => { println!("Unable to write data to output file {} - {}", self.o_file_name, e.kind()); }
        }
      },

      Err(e) => {
        println!("Unable to write output file to {} - {}", self.o_file_name, e.kind());
      }
    }
    
    out
  }

  /**Function sends a simple get request and displays the server repsonse to the screen.
 * Params:
 *  &self
 *  split_wordlist: &str {A chunk of the input wordlist that be handed off to a thread.}
 * Returns JoinHandle<()>
 */
  pub fn thread_get_request(&self, split_wordlist: String) -> thread::JoinHandle<()> {
    let debug = self.debug.clone();
    let debug_dtl = self.debug_dtl.clone();
    let timeout = self.tout.clone();
    
    // Create the thread.
    let handle = thread::spawn(move || {
      let slices: Vec<&str> = split_wordlist.split(" ").collect();        // Create array of slices.

      let mut request = "".to_owned();                            // Builds the request
      for i in slices {
        request.push_str(i);

        match Self::send_get_request(request.as_str(), timeout) {    // Sends the GET reuqest.
          Ok(s) => {
            let status = s.status();
            if status.is_success() {
              println!("{} -- {}", request, status);                     // print OK 200 for successful connections.
            }

            if debug == true {                                           // Enable debugging to print everything.
              println!("{} -- {}", request, status);
              
              if debug_dtl == true {                                     // Enable this flag to get the html body.
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

    /**Function sends a get request to a web server and returns the response.
   * Params:
   *  url:      &str     {The web address to make the request.}
   *  timeout:  u64      {The time in miliseconds before the request is dropped.}
   *  dbg:      bool     {Displays the status code and the html response to the screen.}
   * Returns StatusCode.
   */
  pub fn send_get_request(url: &str, timeout: u64) -> Result<Response, Error> {
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
}

 /**Function checks if a file exists on disk and will create it if not.
  * Params:
    filename: &str {The name of the file.}
  Returns bool.
  */
fn file_exists(file_name: &str) -> bool {
  
  // Closure checks for the existence of a file.
  let check_file = || -> bool {
    let mut out: bool = false;

    match OpenOptions::new().read(true).open(file_name) {
      Ok(_) => { out = true; },
      Err(e) => {
        if e.kind() == ErrorKind::PermissionDenied { println!("Unable to open {} - Permission denied", file_name); }
        else if e.kind() == ErrorKind::NotFound { println!("Unable to open {} - File not found", file_name); }
        else if e.kind() == ErrorKind::InvalidData { println!("Unable to open {} Invalid data [This is not valid utf-8 or utf-16]", file_name); }
        else { println!("Unable to open {} - {}", file_name, e.kind()); }
      }
    }

    out
  };
  
  // Closure creates the file.
  let create_file = || -> bool {
    let mut out: bool = false;

    match OpenOptions::new().read(true).write(true).create(true).open(file_name) {
      Ok(_) => { out = true; },
      Err(e) => {
        println!("Unable to create {} - {}", file_name, e.kind());
      }
    }
    
    out
  };

  let mut flag = check_file();
  if flag == false {
    flag = create_file();
  }

  flag
}

/**Function parses an int and panics if the users input is not a valid digit.
 * Params:
 *  number: &str {The number to be parsed.}
 * Returns u64
 */
fn parse_int(number: &str) -> u64 {
  let mut out: u64 = 0;
  
  match number.parse::<u64>() {
    Ok(s) => { out = s; },
    Err(e) => {
      panic!("This is not an int - {:?}", e.kind());
    }
  }
  
  out
}

/**Function makes get requests depending on the base url and the contents of the supplied wordlist.
 * Params:
 *  set: Settings {The settings or command line arguments that the user supplied.}
 * Returns nothing.
 */
pub fn fuzz_directory(set: Settings<'_>) -> () {

  let file_contents = parse_wordlist(set.wlist);                      // Gets the contents of the wordlist
  if file_contents.1 >= LARGE_FILE {
    println!("Wanring: word list is larger than 50MB. Performance may be slow...");
  }
  
  // Array is split into slice elements 
  let slice_array: Vec<&str> = file_contents.0.split("\r\n").collect();
  let mut temp_string = "".to_owned();                                         // String holds elements to be processed.
  let mut chunk_counter: usize = 0;                                                    // Counts the number of elements.
  let mut handles = vec![];                                       // Stores the thread handles.
  
  let mut url = "".to_owned();
  url.push_str(set.url);

  let sl_array_len = slice_array.len().clone();

  // Allocates memory to string that holds 20 or less elements
  for chunk in slice_array {
    if chunk_counter >= 20 {
      // The string is cloned and passed to the thread.
      let mut c_temp_string = String::new();
      c_temp_string.push_str(temp_string.as_str());

      let test_handle = set.thread_get_request(c_temp_string);
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
    let last_handle = set.thread_get_request(temp_string);
    handles.push(last_handle);
  }

  // Sleep for 2 seconds and join threads into the main thread.
  thread::sleep(Duration::from_secs(2));
  for i in handles { i.join().unwrap(); }
}

/**Function reads file into a buffer and returns it as a string and the total number of bytres that were read.
 * Params:
 *  wlist: &str {The file path and name to the wordlist.}
 * Returns (String, usize)
 */
pub fn parse_wordlist(wlist: &str) -> (String, usize) {

  let mut byte_array = vec![];                                               // Creates vector to stores bytes.
  let read_file = OpenOptions::new().read(true).open(wlist).unwrap();     // Opens file and deals with errors.
  
  let mut reader = BufReader::new(read_file);                 // Buffer is created to read the file. 
  let mut total_bytes_read: usize = 0;

  match reader.read_until(u8::MIN, &mut byte_array) {                      // Reads the entire file and stores in a buffer.
    Ok(b) => { total_bytes_read += b; },
    Err(e) => { println!("{}", e.kind()); }
  }
  
  let string_ptr = String::from_utf8_lossy(&byte_array);                   // Creates a string and removed on utf8 chars.
  let mut utf8_string = String::new();
  match string_ptr {
    Cow::Borrowed(b) => { utf8_string.push_str(b); },
    Cow::Owned(b) => { utf8_string.push_str(b.as_str()); }
  }

  (utf8_string, total_bytes_read)
}

/**Function sends a get request to a web server and returns the response.
 * Params:
 *  url:      &str     {The web address to make the request.}
 *  timeout:  u64      {The time in miliseconds before the request is dropped.}
 *  dbg:      bool     {Displays the status code and the html response to the screen.}
 * Returns StatusCode.
 */
pub fn send_get_request(url: &str, timeout: u64) -> Result<Response, Error> {
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


fn main() {
  let args: Vec<String> = env::args().collect();
  let ln = args.len();
  let mut set = Settings::new();
  let mut list = ListType::Dictonary;
  // let mut output_buffer = "".to_owned();

  // Closure checks mandatory arguments in the following format.
  // {Program.exe <url> <wordlist flag> <wordlist file>}
  // Arguments after the mandatory 4 are optional.
  let mut check_mandatory_args = || -> bool {
    let mut out = true;
    
    // Chcks if http(s) can be found in the url and if the dictonary wordlist actually exists.
    if args[1].contains(F_HTTP) || args[1].contains(F_HTTPS) {
      if args[2].contains(S_DICT) || args[2].contains(L_DICT) {
        // Check if paths exists.

        let path_exists: bool = file_exists(args[3].as_str());
        if path_exists == false {
          println!("Path {} does not exist", args[3].as_str());
          out = path_exists;
        }

        set.url = args[1].as_str();
        set.wlist = args[3].as_str();
        list = ListType::Dictonary;
      }
      else if args[2].contains(S_PARAM) || args[2].contains(L_PARAM) {
        // Check if paths exists.

        let path_exists: bool = file_exists(args[3].as_str());
        if path_exists == false {
          println!("Path {} does not exist", args[3].as_str());
          out = path_exists;
        }

        set.url = args[1].as_str();
        set.wlist = args[3].as_str();
        list = ListType::Parameter;
      }
    }
    else {
      println!("The url must start with either http:// or https://");
      out = false;
    }

    out
  };

  // Checks if optional arguments after the mandatory 4 are valid.
  let mut check_opt_args = || -> bool {
    if ln < 5 { return false; }
    let mut out: bool = true;

    // Block below will check if the arguments entered do not match any of the valid arguments.
    let check_valid_args = |index: &str| -> bool {
      // let mut out = false;
      
      if !index.contains(S_OUT) || !index.contains(S_TIME) || !index.contains(F_DEBUG) 
        || !index.contains(L_OUT) || !index.contains(L_TIME) || !index.contains(F_DEBUG_DTL) {
        return false;
      }

      // match index {
      //   S_OUT => { out = true; }
      //   L_OUT => { out = true; }
      //   S_TIME => { out = true; }
      //   L_TIME => { out = true; }
      //   F_DEBUG => { out = true; }
      //   F_DEBUG_DTL => { out = true; }
      //   _ => {}
      // }
      
      return out;
    };

    for i in 4..ln {

      println!("{}", args[i]);
      // This check should only fail if no filename is supplied or if the file cannot be created.
      if args[i].contains(S_OUT) || args[i].contains(L_OUT) {
        if file_exists(args[i+1].as_str()) == false {
          println!("{} does not exist or could not be written to disk.", args[i+1].as_str());
          out = false;
          break;
        }

        set.o_file_name = args[i+1].as_str();
      }

      // Check will only fail if the timeout provided is not an actual int.
      else if args[i].contains(S_TIME) || args[i].contains(L_TIME) {
        let test_value: u64 = parse_int(args[i+1].as_str());

        if test_value == 0 {
          println!("You have either entered input that is not a number or a test has failed.\nDefaulting to 50 miliseconds.");
        }

        set.tout = test_value;
        break;
      }

      // Check does nothing for now.
      else if args[i].as_str().contains(F_DEBUG) || args[i].as_str().contains(F_DEBUG_DTL) {

        // The match statement is used to get exact matches on similar values.
        match args[i].as_str() {
          F_DEBUG => { set.debug = true; }
          F_DEBUG_DTL => {
            set.debug = true;
            set.debug_dtl = true;
          }
          _ => { println!("{}", SYNTAX); }
        }

      }

      // Checks if each arg is valid.
      else if check_valid_args(args[i].as_str()) == false {
        out = false;
        break;
      }
    
    }

    // Set the timeout to the default if still zero by the end of the loop.
    if set.tout == 0 { set.tout = DF_TIMEOUT; }

    out
  };

  // Closure checks if syntax checks passed or failed.
  let result_args = |result: bool, r_type: &str| -> () {
    if result == false {
      println!("{} args Failed", r_type);
      return;
    }
    else if result == true {
      println!("{} args passed", r_type);
      return;
    }
  };

  // Displays the help screen.
  let show_syntax = || -> () {
    println!("{}", SYNTAX);
    exit(0);
  };

  match ln {
    1  => {
      show_syntax();
    }
    2  => {
      if args[1].contains(S_HELP) || args[1].contains(L_HELP) {
        show_syntax();
      }
      else {
        show_syntax();
      }
    }
    3  => {
      show_syntax();
    }
    4  => {
      let result = check_mandatory_args();
      set.debug = true;

      if set.debug == true {
        set.show_information();
        result_args(result, "Mandatory");
        println!("ListType=[{:?}]", list);
      }
      
      if list == ListType::Dictonary { fuzz_directory(set); }
      else                           { /*fuzz_parameter(set);*/ }

      return;
    }
    5 => {
      let man = check_mandatory_args();
      let opt = check_opt_args();
      
      if set.debug == true {
        result_args(man, "Mandatory");
        result_args(opt, "Optional");
        set.show_information();
        println!("ListType=[{:?}]", list);
      }
      
      if list == ListType::Dictonary { fuzz_directory(set); }
      else                           { /*fuzz_parameter(set);*/ }

      return;
    }
    6 => {
      let man = check_mandatory_args();
      let opt = check_opt_args();
      
      if set.debug == true {
        result_args(man, "Mandatory");
        result_args(opt, "Optional");
        set.show_information();
        println!("ListType=[{:?}]", list);
      }
      
      if list == ListType::Dictonary { fuzz_directory(set); }
      else                           { /*fuzz_parameter(set);*/ }

      return;
    }
    7 => {
      let man = check_mandatory_args();
      let opt = check_opt_args();
      
      if set.debug == true {
        result_args(man, "Mandatory");
        result_args(opt, "Optional");
        set.show_information();
        println!("ListType=[{:?}]", list);
      }
      
      if list == ListType::Dictonary { fuzz_directory(set); }
      else                           { /*fuzz_parameter(set);*/ }

      return;
    }
    8 => {
      let man = check_mandatory_args();
      let opt = check_opt_args();
      
      if set.debug == true {
        result_args(man, "Mandatory");
        result_args(opt, "Optional");
        set.show_information();
        println!("ListType=[{:?}]", list);
      }
      
      if list == ListType::Dictonary { fuzz_directory(set); }
      else                           { /*fuzz_parameter(set);*/ }

      return;
    }
    9 => {
      let man = check_mandatory_args();
      let opt = check_opt_args();
      
      if set.debug == true {
        result_args(man, "Mandatory");
        result_args(opt, "Optional");
        set.show_information();
        println!("ListType=[{:?}]", list);
      }
      
      if list == ListType::Dictonary { fuzz_directory(set); }
      else                           { /*fuzz_parameter(set);*/ }

      return;
    }
    10 => {
      let man = check_mandatory_args();
      let opt = check_opt_args();
      
      if set.debug == true {
        result_args(man, "Mandatory");
        result_args(opt, "Optional");
        set.show_information();
        println!("ListType=[{:?}]", list);
      }
      
      if list == ListType::Dictonary { fuzz_directory(set); }
      else                           { /*fuzz_parameter(set);*/ }

      return;
    }
    _  => { show_syntax(); }
  }
}
