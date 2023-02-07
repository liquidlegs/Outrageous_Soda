use clap::Parser;
use std::{
  io::{Write, BufReader, BufRead, Error as IoError, ErrorKind},
  fs::OpenOptions,
  thread,
  thread::JoinHandle,
  borrow::Cow,
  sync::mpsc::{Sender, Receiver},
  sync::mpsc,
};

use core::time::Duration;
use console::style;

use reqwest::{
  self, StatusCode, Error,
  blocking::Response,
};

mod fixed_buffer;
use fixed_buffer::u8::U8FixedBuffer;

pub const LARGE_FILE: usize = 50000000;                 // Displays warning for files larger than 50 MB.
pub static F_HTTP: &str = "http://";                    // Checks if http:// is in the url. 
pub static F_HTTPS: &str = "https://";                  // Checks if http:// is in the url.
pub const WIN_NEW_LINE: &str = "\r\n";                  // The Windows style new line.
pub const LNX_NEW_LINE: &str = "\n";                    // The Linux style new line.
pub const TITLE : &str = "
___        _                                            ____            _       
/ _ \\ _   _| |_ _ __ __ _  __ _  ___  ___  _   _ ___    / ___|  ___   __| | __ _ 
| | | | | | | __| '__/ _` |/ _` |/ _ \\/ _ \\| | | / __|   \\___ \\ / _ \\ / _` |/ _` |
| |_| | |_| | |_| | | (_| | (_| |  __/ (_) | |_| \\__ \\    ___) | (_) | (_| | (_| |
\\___/ \\__,_|\\__|_|  \\__,_|\\__, |\\___|\\___/ \\__,_|___/___|____/ \\___/ \\__,_|\\__,_|
                          |___/                    |_____|                                           
";

#[derive(Debug, Parser)]
#[clap(author = "liquidlegs", version = "0.1.0", about, help = "")]
pub struct SodaArgs {
  /// Url
  #[clap(value_parser)]
  pub url: String,

  /// WordList
  #[clap(value_parser)]
  pub wordlist: String,

  /// Fuzz
  #[clap(value_enum)]
  pub fuzz: Fuzz,

  /// Debug
  #[clap(long, default_value_if("debug", Some("false"), Some("true")), min_values(0))]
  pub debug: bool,

  /// Debug Detail
  #[clap(short, long, default_value_if("verbose", Some("false"), Some("true")), min_values(0))]
  pub verbose: bool,

  /// Html Response
  #[clap(short = 'H', long, default_value_if("htmlbody", Some("false"), Some("true")), min_values(0))]
  pub htmlbody: bool,

  /// File Extensions
  #[clap(short, long)]
  pub ext: Option<String>,

  /// Output file
  #[clap(short, long)]
  pub output: Option<String>,

  #[clap(short, long = "scodes")]
  /// Status codes
  pub status_codes: Option<String>,

  /// Timeout (miliseconds)
  #[clap(short, long, default_value = "300")]
  pub timeout: u64,

  /// Threads
  #[clap(short = 'T', long, default_value = "10")]
  pub threads: usize,
}

pub fn display_help() -> () {
  println!(
    "{}
    
    {}:
        {} <URL> <WORD_LIST> <FUZZ> [OPTIONS]
    
    {}:
        <URL>     The base url in the GET request
        <FILE>    A wordlist used for generating GET requests
        <FUZZ>    Fuzz a URI path or paramater [possible values: directory-path, parameter]
    
    {}:
            --{}                     Shows error messages and all server responses
        {}, --{}         <EXT[...]>    Generate testcases by a comma,separated,list,of,extensions
        {}, --{}                  Show html responses
        {}, --{}                      Print help information
        {}, --{}      <FILE>        Output results to a file
        {}, --{}     <INT>         The timeout period before the connection is dropped in miliseconds - [default: 300]
        {}, --{}     <INT>         The number of threads you wish to use to process requests - [default:10]
        {}, --{}      <CODES[...]>  Specify the status codes to be displayed - [default: 200]
        {}, --{}                   Show all status codes
        {}, --{}                   Print version information
        
    {}:
        {} \"{}//127.0.0.1/?{}={{!}}&{}=hacked\" C:\\folder\\wordlist.txt {} --{} {} {}
        {} \"{}//127.0.0.1/files\" C:\\directory\\rockyou.txt {} {} {} {} --{} {}",
      
      style(TITLE).red().bright(), style("USAGE").yellow().bright(), style("Outraegeous_Soda.exe").red().bright(), 
      style("ARGS").yellow().bright(), style("OPTIONS").yellow().bright(),
      style("debug").cyan(), style("-e").green().bright(), style("ext").cyan(), style("-H").green().bright(), 
      style("htmlbody").cyan(), style("-h").green().bright(), style("help").cyan(), style("-o").green().bright(), style("output").cyan(), 
      style("-t").green().bright(), style("timeout").cyan(), style("-T").green().bright(), style("threads").cyan(), 
      style("-s").green().bright(), style("scodes").cyan(), style("-v").green().bright(), style("verbose").cyan(), style("-V").green().bright(), 
      style("version").cyan(), style("EXAMPLES").yellow().bright(),
      style("outraegeous_soda.exe").red().bright(), style("http:").yellow(), style("username").cyan(), style("password").magenta().bright(),
      style("parameter").magenta().bright(), style("debug").cyan(), style("-T").green().bright(), style("30").yellow(),
      style("outraegeous_soda.exe").red().bright(), style("http:").yellow(), style("directory-path").magenta().bright(),
      style("-H").green().bright(), style("-T").green().bright(), style("15").yellow(), style("timeout").cyan(),
      style("1000").yellow());
}

#[derive(Debug, Clone)]
pub enum ThreadMessage {
  Finished,
  Continue,
}

pub mod arg_fmt {
  use console::style;

  pub fn f_debug(msg: &str, value: &str) -> () {
    println!("{} {} {}", style("Debug =>").red().bright(), style(msg).yellow(), style(value).cyan());
  }

  pub fn f_error(msg: &str, value: &str, error_enum: String) -> () {
    println!("{}: {} {} - {}", style("Error").red().bright(), msg, style(value).cyan(),style(error_enum).red());
  }

  pub fn f_io(bytes: usize, value: &str) -> () {
    println!("{}: {} bytes were successfully written to {}", style("Ok").yellow().bright(), style(bytes).cyan(), style(value).cyan());
  }
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

  pub fn compare_status_code(request: &String, status: StatusCode) -> () {
    match status {
      StatusCode::OK =>                        { println!("{request} -- {}", style(status).green().bright()); }
      StatusCode::ACCEPTED =>                  { println!("{request} -- {}", style(status).green().bright()); }
      StatusCode::BAD_GATEWAY =>               { println!("{request} -- {}", style(status).red().bright());   }
      StatusCode::BAD_REQUEST =>               { println!("{request} -- {}", style(status).red().bright());   }
      StatusCode::EXPECTATION_FAILED =>        { println!("{request} -- {}", style(status).red().bright());   }
      StatusCode::FAILED_DEPENDENCY =>         { println!("{request} -- {}", style(status).red().bright());   }
      StatusCode::HTTP_VERSION_NOT_SUPPORTED =>{ println!("{request} -- {}", style(status).red().bright());   }
      StatusCode::NOT_FOUND =>                 { println!("{request} -- {}", style(status).red().bright());   }
      StatusCode::INTERNAL_SERVER_ERROR =>     { println!("{request} -- {}", style(status).red().bright());   }
      StatusCode::GATEWAY_TIMEOUT =>           { println!("{request} -- {}", style(status).red().bright());   }
      _ =>                                     { println!("{request} -- {}", style(status).cyan());           }
    }
  }

  /**Function sends a simple get request and displays the server repsonse to the screen.
 * Params:
 *  &self
 *  split_wordlist: &str {A chunk of the input wordlist that be handed off to a thread.}
 * Returns JoinHandle<()>
 */
  pub fn thread_get_request(&self, split_wordlist: String, sender: Sender<ThreadMessage>) -> thread::JoinHandle<()> {
    let debug = self.debug.clone();
    let verbose = self.verbose.clone();
    let timeout = self.timeout.clone();
    let html = self.htmlbody.clone();
    let mut output = "".to_owned();
    let status_codes = self.get_status_codes();

    match self.output.clone() {
      Some(s) => { output.push_str(s.as_str()); },
      None => {}
    }
    
    // Create the thread.
    let handle = thread::spawn(move || {
      let c_sender = sender.clone();
      let mut u8_buffer = U8FixedBuffer::new();            // Stores data to be logged.

      let slices: Vec<&str> = split_wordlist.split(" ").collect();        // Create array of slices.
      if debug == true {
        println!("{} {} {} {}", style("Debug =>").red().bright(), 
        style("Thread cycling through").yellow(), style(slices.len().clone()).cyan(), style("test cases\n").yellow());
      }

      let mut request = "".to_owned();                            // Builds the request
      for i in slices {
        
        if u8_buffer.len >= u8_buffer.cap-200 {                           // Buffer is emptied and written to disk.
          match u8_buffer.write_data(output.as_str()) {
            Ok(s) => {
              if debug == true {
                arg_fmt::f_io(s, output.as_str());
              }
            },
            Err(e) => {
              arg_fmt::f_error("Failed to write data to file", "", format!("{}", e.kind()));
            }
          }

          u8_buffer.clear();
        }
        
        request.push_str(i);

        match Self::get(request.as_str(), timeout) {                 // Sends the GET reuqest.
          Ok(s) => {
            let status = s.status();

            if debug == false {
              for i in status_codes.clone() {
                if status.clone() == i {
                  Self::compare_status_code(&request, i);
                  
                  if output.len().clone() > 0 {                              
                    u8_buffer.push_str(format!("{} -- {}\n", request, status).as_str());
                  }
                }
              }
            }

            if verbose == true {                                         // Enable debugging to print everything.
              Self::compare_status_code(&request, status);  
            }

            if html == true {                                            // Enable this flag to get the html body.
              match s.text() {
                Ok(body) => {
                  println!("|\n|\n{}", body);
                }
                Err(e) => { println!("{}", e); }
              }
            }

            match sender.send(ThreadMessage::Continue) {
              Ok(_) => {},
              Err(e) => {
                println!("{e}");
              }
            }
            
          },
          Err(e) => {
            if e.is_builder() != true {
              println!("\n{}\n__________________________________________________", e);
            }

            match sender.send(ThreadMessage::Continue) {
              Ok(_) => {},
              Err(e) => {
                println!("{e}");
              }
            }
          }
        }

        request.clear();
        thread::sleep(Duration::from_millis(100));
      }

      // The contents of the u8 buffer is written to disk if there are left overs after completing the loop.
      if u8_buffer.len > 0 {                                                                
        match u8_buffer.write_data(output.as_str()) {
          Ok(s) => {
            arg_fmt::f_io(s, output.as_str());
          },
          Err(e) => { arg_fmt::f_error("Failed to write data to file", "", format!("{}", e.kind())); }
        }

        u8_buffer.clear();
      }

      match sender.send(ThreadMessage::Finished) {
        Ok(_) => {},
        Err(e) => {
          println!("{e}");
        }
      }
    });

    if debug == true {
      arg_fmt::f_debug("Starting thread", format!("{:?}", handle.thread().id()).as_str());
    }

    handle
  }

  pub fn standard_get_request(&self, split_wordlist: String) -> () {
    let debug = self.debug.clone();
    let verbose = self.verbose.clone();
    let timeout = self.timeout.clone();
    let html = self.htmlbody.clone();
    let mut output = "".to_owned();
    let status_codes = self.get_status_codes();

    match self.output.clone() {
      Some(s) => { output.push_str(s.as_str()); },
      None => {}
    }
    
    let mut u8_buffer = U8FixedBuffer::new();            // Stores data to be logged.
    let slices: Vec<&str> = split_wordlist.split(" ").collect();        // Create array of slices.
    if debug == true {
      println!("Thread cycling through {} test cases\n", slices.len().clone());
    }

    let mut request = "".to_owned();                            // Builds the request.
    for i in slices {
      
      if u8_buffer.len >= u8_buffer.cap-200 {                           // Fixed buffer is emptied and written to disk.
        match u8_buffer.write_data(output.as_str()) {
          Ok(s) => {
            if debug == true {
              arg_fmt::f_io(s, output.as_str());
            }
          },
          Err(e) => {
            if debug == true {
              arg_fmt::f_error("Failed to write data to file", "", format!("{}", e))
            }
          }
        }

        u8_buffer.clear();
      }
      
      request.push_str(i);

      match Self::get(request.as_str(), timeout) {                 // Sends the GET reuqest.
        Ok(s) => {
          let status = s.status();
          
          if debug == false {
            for i in status_codes.clone() {
              if status.clone() == i {
                Self::compare_status_code(&request, i);
                
                if output.len().clone() > 0 {                              
                  u8_buffer.push_str(format!("{} -- {}\n", request, status).as_str());
                }
              }
            }
          }

          if verbose == true {                                         // Enable debugging to print everything.
            Self::compare_status_code(&request, status);  
          }

          if html == true {                                            // Enable this flag to get the html body.
            match s.text() {
              Ok(body) => {
                println!("|\n|\n{}", body);
              }
              Err(e) => { println!("{}", e); }
            }
          }
        },
        Err(e) => {
          if e.is_builder() != true {
            println!("\n{}\n__________________________________________________", e);
          }
        }
      }

      request.clear();
    }

    // The contents of the u8 buffer is written to disk if there are left overs after completing the loop.
    if u8_buffer.len > 0 {
      match u8_buffer.write_data(output.as_str()) {
        Ok(s) => { println!("{} bytes were successfully written to  {}", s, output.as_str()); },
        Err(e) => { println!("Failed data to file with error: {}", e.kind()); }
      }

      u8_buffer.clear();
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

        match reader.read_until(u8::MIN, &mut byte_array) {                  // Reads the entire file and stores in a buffer.
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
  #[allow(unused_assignments)]
  pub fn begin_fuzz(&self) -> () {
    let mut pattern = "";
    let fuzz_type = self.fuzz.clone();
    
    let file_contents = self.parse_wordlist();                                   // Gets the contents of the wordlist
    if file_contents.1 >= LARGE_FILE {
      println!(
        "{}: {} {}", 
        style("Warning").yellow().bright(), style("word-list is larger than 50MB.").cyan(),
        style("Performance may be slow...").red().bright()
      );
    }

    // The next 30 lines from here works whether the text file using the windows \r\n new line or the linux \n new line
    let mut win_test_string = String::new();
    let mut lnx_test_string = String::new();
    if file_contents.1 < 256 {
      win_test_string.push_str(&file_contents.0[0..file_contents.1]);                     // Creates a string slice smaller than 256 bytes.
    }
    else {
      win_test_string.push_str(&file_contents.0[0..256]);                                 // Creates a string slice with no more than 256 bytes.
    }

    if self.check_correct_split(win_test_string, WIN_NEW_LINE) == true {
      pattern = WIN_NEW_LINE;
      drop(lnx_test_string);
    }

    else {
      if file_contents.1 < 256 { lnx_test_string.push_str(&file_contents.0[0..file_contents.1]); }
      else                     { lnx_test_string.push_str(&file_contents.0[0..256]);             }
      
      if self.check_correct_split(lnx_test_string, LNX_NEW_LINE) == true {
        pattern = LNX_NEW_LINE;
      }
      else {
        println!("{}: Expected windows (\\r\\n) or Linux (\\n) new line delimiter.", style("Error").red().bright());
        return;
      }
    }

    let mut ext_string_len: usize = 0;
    let mut ext_string = String::new();
    
    if fuzz_type == Fuzz::DirectoryPath {
      match self.ext.clone() {
        Some(s) => {
          ext_string.push_str(s.as_str());
          ext_string_len = ext_string.len();
        },
        None => {}
      }
    }
    
    // Array is split into slice elements 
    let slice_array: Vec<&str> = file_contents.0.split(pattern).collect();
    let mut temp_string = "".to_owned();                                                 // String holds elements to be processed.
    let mut replace_string = "".to_owned();
    let mut chunk_counter: usize = 0;                                                            // Counts the number of elements.
    let mut handles = vec![];                                               // Stores the thread handles.
    let mut empty_string: usize = 0;
    
    if self.threads == 0 {
      empty_string = 20;
    }
    else if self.threads > 0  {
      empty_string = slice_array.len().clone() as usize / self.threads;
    }
     
    let url = self.url.as_str();
    let sl_array_len = slice_array.len().clone();

    if fuzz_type == Fuzz::DirectoryPath {
      if ext_string_len == 0 {
        println!(
          "{} {} {}", style("Generating").yellow(),  
          style(sl_array_len.clone()).cyan(), style("test cases...").yellow()
        );
      }
      
      else {
        println!(
          "{} {} {}", style("Generating").yellow(),  
          style(sl_array_len.clone()*ext_string_len.clone()).cyan(), style("test cases...").yellow()
        );
      }
    }

    if fuzz_type == Fuzz::Parameter {
      println!(
        "{} {} {}", style("Generating").yellow(),  
        style(sl_array_len.clone()).cyan(), style("test cases...").yellow()
      );
    }

    thread::sleep(Duration::from_secs(1));
    let (sender, recv) = mpsc::channel::<ThreadMessage>();

    // Allocates memory to string that holds 20 or less elements
    for chunk in slice_array {
      if chunk_counter >= empty_string {
        // The string is cloned and passed to the thread.
        let mut c_temp_string = String::new();
        c_temp_string.push_str(temp_string.as_str());
        let c_sender = sender.clone();

        if self.threads == 0 {
          self.standard_get_request(c_temp_string);
        }
        else if self.threads > 0 {
          let test_handle = self.thread_get_request(c_temp_string, c_sender);
          handles.push(test_handle);
        }

        temp_string.clear();
        chunk_counter = 0;
      }
      
      // Setup each element and push them to the string.
      if fuzz_type == Fuzz::DirectoryPath {
        
        if ext_string_len > 0 {
          let exts: Vec<&str> = ext_string.split(",").collect();
          
          for i in exts {                         
            temp_string.push_str(url);          // https://address
            temp_string.push_str(chunk);        // + word
            temp_string.push('.');                  // + '.'
            temp_string.push_str(i);            // + extension
            temp_string.push(' ');                  // + ' '
          }
  
        }
        else {
          temp_string.push_str(url);
          temp_string.push_str(chunk);
        }

      }
      else if fuzz_type == Fuzz::Parameter {
        replace_string = url.replace("{!}", chunk);
        temp_string.push_str(replace_string.as_str());
      }

      
      if chunk_counter < sl_array_len {
        temp_string.push(' ');
      }

      chunk_counter += 1;
    }

    // Run thread for left over elements that did not exceed past 20.
    if temp_string.len() > 0 {
      let last_handle = self.thread_get_request(temp_string, sender);
      handles.push(last_handle);
    }

    if self.threads > 0 {
      println!("{}: {}", style("OK").yellow().bright(), style("Waiting on threads...").cyan());

      // Sleep for 1 second and join threads into the main thread.
      let mut finished_threads: Vec<ThreadMessage> = Default::default();
      let mut retry_counter: usize = 0;

      // The code block belows ensures threads are joined to the main thread when their finished.
      loop {
        match recv.recv_timeout(Duration::from_millis(100)) {
          Ok(s) => {

            match s {
              ThreadMessage::Continue => {}
              ThreadMessage::Finished => {
                finished_threads.push(s);
              }
            }

          },
          Err(e) => {
            println!("{e}");
            
            if retry_counter >= 10 {
              break;
            }
            
            if self.debug.clone() == true {
              arg_fmt::f_debug("retrying to receive message from thread", retry_counter.to_string().as_str());
            }

            retry_counter += 1;
          }
        }

        if finished_threads.len() >= self.threads.clone()+1 {
          break;
        }
      }
      

      for i in handles {
        let id = i.thread().id().clone();
        
        match i.join() {
          Ok(_) => {
            if self.debug.clone() == true {
              println!(
                "{} {} {:?}", 
                style("Debug =>").red().bright(), style("joining thread to the main thread").yellow(),
                style(id).cyan()
              );
            }
          },
          Err(e) => {}
        }
      } 
    }

    println!("Done!");
  }

  // /**Function waits on a thread to finish before joining the output into the main thread.
  //  * Params:
  //  *  handle: JoinHandle<()> {The handle to thread.}
  //  *  debug:  bool           {Display information about threads if enabled.}
  //  * Returns bool.
  //  */
  // #[allow(unused_assignments)]
  // pub fn wait_on_threads(handle: JoinHandle<()>, debug: bool) -> bool {
  //   let mut out = false;
  //   let mut time_counter: usize = 0;

  //   loop {
  //     if time_counter >= 60 { time_counter = 0; }
      
  //     if handle.is_finished() == true {
  //       out = true;
  //       break;
  //     }

  //     thread::sleep(Duration::from_secs(1));
  //     time_counter += 1;
      
  //     if debug == true && time_counter >= 60 { println!("Waiting on threads..."); }
  //   }

  //   match handle.join() {
  //     Ok(_) => {}
  //     Err(_) => {}
  //   }

  //   if debug == true { println!("thread finished"); }
    
  //   out
  // }

  /**Function displays 256 bytes of the wordlist before it has been split into an array and after.
   * Params:
   *  &self
   * Returns nothing.
  */
  #[allow(unused_assignments)]
  pub fn dbg_print_chunk(&self) -> () {
    let file_contents = self.parse_wordlist();
    let mut slice = "";

    if file_contents.1 < 256 { slice = &file_contents.0[0..file_contents.0.len()]; }
    else                     { slice = &file_contents.0[0..256]; }

    println!("{:?}", slice);

    let mut win_test_string = String::new();
    win_test_string.push_str(slice);

    let mut lnx_test_string = String::new();
    lnx_test_string.push_str(slice);

    let win_slice_array: Vec<&str> = win_test_string.split(WIN_NEW_LINE).collect();
    println!("{:?}", win_slice_array);

    let lnx_slice_array: Vec<&str> = lnx_test_string.split(LNX_NEW_LINE).collect();
    println!("{:?}", lnx_slice_array);
  }

  /**Function checks if a string has been correctly split.
   * Params:
   *  &self
   *  split_string: String {The string to be used in the used.}
   *  pattern:      &str   {The delimiter for splitting the string.}
   * Returns bool.
   */
  pub fn check_correct_split(&self, split_string: String, pattern: &str) -> bool {
    let mut out = false;
    let slice_array: Vec<&str> = split_string.split(pattern).collect();
    
    if slice_array.len() > 1 {
      out = true;
    }
    
    out
  }

  pub fn get_status_codes(&self) -> Vec<StatusCode> {
    let mut values: Vec<u16> = Default::default();
    let mut codes = String::from("");
    
    if let Some(c) = self.status_codes.clone() {
      codes.push_str(c.as_str());
    }

    let split_success = self.check_correct_split(codes.clone(), ",");
    if split_success == true {
      let split: Vec<&str> = codes.split(",").collect();
      
      for i in split {
        match i.parse::<u16>() {
          Ok(s) => { values.push(s); },
          Err(e) => {
            panic!(
              "{}: unable to parse status code {} - {}", 
              style("Error").red().bright(), style(i).cyan(), style(e).red().bright()
            );
          }
        }
      }
    }

    else {
      if codes.len() > 0 {
        let test_value = codes.as_str();
        
        match test_value.parse::<u16>() {
          Ok(s) => { values.push(s); },
          Err(e) => {
            panic!(
              "{}: unable to parse status code {} - {}", 
              style("Error").red().bright(), style(test_value).cyan(), style(e).red().bright()
            );
          }
        }
      }
    }

    let mut out: Vec<StatusCode> = Default::default();
    for i in values {
      match StatusCode::from_u16(i) {
        Ok(s) => {
          out.push(s);
        },
        Err(e) => {}
      }
    }

    out
  }
}