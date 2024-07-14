use std::ffi::OsStr;
use std::path::Path;
use std::process::exit;

use clap::Arg;
use clap::command;
use inquire::Text;
use inquire::validator::Validation;

fn handle_interactive() -> (String, String) {
   // fuck you inquire for using some fucking weird ass result type
   // duplicating code but it is what it is
   let file_exists = |input: &str| {
      let path = Path::new(input);
      if !path.exists() {
         return Ok(Validation::Invalid("File does not exist.".into()));
      }
      if !path.is_file() {
         return Ok(Validation::Invalid("Path is not a file.".into()));
      }
      Ok(Validation::Valid)
   };
   
   let valid_extension = |input: &str| {
      let path = Path::new(input);
      let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
      let valid = ["json", "yaml", "yml", "xml"];
      if valid.contains(&ext) {
         Ok(Validation::Valid)
      } else {
         let msg = format!("Incorrect file path, must be one of {valid:?}");
         Ok(Validation::Invalid(msg.into()))
      }
   };

   let input_path = Text::new("Input file: ")
      .with_validator(file_exists)
      .with_validator(valid_extension)
      .prompt();
   if input_path.is_err() {
      exit(1);
   }
   let input_path = input_path.unwrap();

   let output_path = Text::new("Output file: ")
      .with_validator(valid_extension)
      .prompt();
   if output_path.is_err() {
      exit(1);
   }
   let output_path = output_path.unwrap();

   (input_path, output_path)
}

fn main() {
   let file_exists = move |input: &str| -> Result<String, String> {
      let path = Path::new(input);
      if !path.exists() {
         return Err("File does not exist.".into());
      }
      if !path.is_file() {
         return Err("Path is not a file.".into());
      }
      Ok(input.into())
   };

   let valid_extension = move |input: &str| -> Result<String, String> {
      let path = Path::new(input);
      let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
      let valid = ["json", "yaml", "yml", "xml"];
      if valid.contains(&ext) {
         Ok(input.into())
      } else {
         Err(format!("Incorrect file path, must be one of {valid:?}"))
      }
   };

   let input_parser = move |input: &str| -> Result<String, String> {
      let exists = file_exists(input);
      exists?;
      valid_extension(input)
   };

   let matches = command!()
      .disable_version_flag(true)
      .arg(
         Arg::new("input")
            .value_parser(input_parser)
            .help("Path to the input file")
            .requires("output")
      ).arg(
         Arg::new("output")
            .value_parser(valid_extension)
            .help("Path to the output file")
      ).get_matches();

   let input = matches.get_one::<String>("input");
   
   let (input_path, output_path) = if let Some(input_arg) = input {
      (
         input_arg.clone(),
         matches.get_one::<String>("output").unwrap().clone()
      )
   } else {
      handle_interactive()
   };

   println!("input: {:?}", input_path);
   println!("output: {:?}", output_path);
}
