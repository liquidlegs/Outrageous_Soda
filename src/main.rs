#[path = "arguments\\mod.rs"]
pub mod arguments;
use crate::arguments::*;
use clap::Parser;

fn main() -> () {
  let mut args = SodaArgs::parse();

  if args.debug == true {
    args.show_information();
    args.dbg_print_chunk();
  }

  if !args.url.contains(F_HTTP) || args.url.contains(F_HTTPS) {
    println!("url must start with {} or {}", F_HTTP, F_HTTPS);
    return;
  }
  else {
    if args.url.as_str().chars().last().unwrap() != '/' {
      args.url.push('/');
    }
  }

  if args.fuzz == Fuzz::DirectoryPath {
    let check_file = SodaArgs::file_exists(args.wordlist.as_str());

    if check_file == false {
      println!("Error: {} does not exist", args.wordlist.as_str());
      return;
    }

    args.fuzz_directory();
  }

  else if args.fuzz == Fuzz::Parameter {

  }
}