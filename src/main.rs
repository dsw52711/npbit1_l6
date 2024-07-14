use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::io::Error as IOError;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use clap::Arg;
use clap::command;
use inquire::Text;
use inquire::validator::Validation;
use serde_json::Value as JsonValue;

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

fn read_input_data(input_path: &str) -> Result<String, IOError> {
   let mut input_file = File::open(input_path)?;
   let mut input_data = String::new();
   input_file.read_to_string(&mut input_data)?;
   Ok(input_data)
}

fn json_to_value(json_data: &str) -> Result<JsonValue, String> {
   let value = serde_json::from_str(json_data);
   if let Err(err) = value {
      return Err(err.to_string());
   }
   Ok(value.unwrap())
}

fn value_to_json(value: &JsonValue) -> Result<String, String> {
   let json_data = serde_json::to_string_pretty(value);
   if let Err(err) = json_data {
      return Err(err.to_string());
   }
   Ok(json_data.unwrap())
}

fn convert_data(input_path: &str, output_path: &str) -> Result<String, String> {
   let input_data = read_input_data(input_path);
   if let Err(err) = input_data {
      return Err(err.to_string());
   }
   let input_data = input_data.unwrap();

   let input_format = Path::new(input_path).extension().unwrap().to_str().unwrap();
   let output_format = Path::new(output_path).extension().unwrap().to_str().unwrap();

   let output_data = match input_format {
      "json" => {
         let value = json_to_value(&input_data)?;
         match output_format {
            "json" => value_to_json(&value).unwrap(),
            _ => unreachable!()
         }
      },
      _ => unreachable!()
   };

   Ok(output_data)
}

fn save_data(data: &str, output_path: &str) -> Result<(), String> {
   let output_file = File::create(Path::new(output_path));
   if let Err(err) = output_file {
      return Err(err.to_string());
   }
   let mut output_file = output_file.unwrap();
   let result = output_file.write_all(data.as_bytes());
   if let Err(err) = result {
      return Err(err.to_string());
   }
   Ok(())
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

   let input_parser = move |input: &str| {
      file_exists(input)?;
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

   let converted = convert_data(&input_path, &output_path).unwrap();
   let res = save_data(&converted, &output_path);
   if let Err(err) = res {
      eprintln!("Conversion failed:\n{}", err);
      exit(1);
   }
   println!("Conversion successful");

   println!("input: {:?}", input_path);
   println!("output: {:?}", output_path);
}
