#[path = "arguments\\mod.rs"]
pub mod arguments;
use crate::arguments::*;
use arguments::display_help;
use clap::Parser;

fn main() -> () {
  let env_arg: Vec<String> = std::env::args().collect();
  match env_arg[1].as_str() {
    "--help" | "-h" => {
      display_help();
    }
    _ => {}
  }

  // Here we parse all the command line arguments.
  let mut args = SodaArgs::parse();

  if args.debug == true {
    args.show_information();      // Shows the contents of the SodaArgs structure,
    args.dbg_print_chunk();       // Displays how the file is being parsed.
  }

  // Returns if urls dont start with http(s)://
  if !args.url.contains(F_HTTP) || args.url.contains(F_HTTPS) {
    println!("url must start with {} or {}", F_HTTP, F_HTTPS);
    return;
  }

  // Adds a slash to the end of urls where not present.
  else {
    if args.fuzz == Fuzz::DirectoryPath && args.url.as_str().chars().last().unwrap() != '/' {
      args.url.push('/');
    }
  }

  // Creates if the output file is not already created.
  match args.output.clone() {
    Some(str) => {
      if SodaArgs::file_exists(str.as_str()) == false {
        
        match SodaArgs::create_file(str.as_str()) {
          Ok(_) => {
            if args.verbose.clone() == true {
              println!("Successfully created output file at {}", str.as_str());
            }
          },

          Err(e) => {
            if args.debug.clone() == true {
              println!("Unable to create output file at {} with error: {}", str.as_str(), e.kind());
            }
          }
        }
      }
    },
    None => {}
  }

  // Checks if the wordlist exists and returns if not.
  let check_file = SodaArgs::file_exists(args.wordlist.as_str());

  if check_file == false {
    println!("Error: {} does not exist", args.wordlist.as_str());
    return;
  }

  // Fuzzing starts here.
  args.begin_fuzz();
}