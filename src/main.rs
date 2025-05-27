use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{
    fs,
    io::{Read, Write},
};

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    user_name: String,
    tasks: Vec<String>,
    completed: Vec<String>,
}

fn init() -> std::io::Result<()> {
    let mut file = fs::File::create_new("user_data.txt")?;
    let data_default = r#"
{
  "user_name": "ExampleUser",
  "tasks": [
    "Buy groceries",
    "Finish report",
    "Call John"
  ],
  "completed": [
    "Walk the dog",
    "Read a book"
 ]
}
"#;

    file.write_all(data_default.as_bytes())
        .expect("unable to write to the file : ");
    Ok(())
}

fn decerialize_file() -> Data {
    let user_data = fs::read_to_string("user_data.txt")
        .unwrap_or_else(|err| panic!("unable to read file : {err:?}"));
    let data: Data = serde_json::from_str(user_data.as_str())
        .unwrap_or_else(|err| panic!("unable to decerialize the file , : {err:?}"));
    data
}

fn main() {
    //let user_file;
    if init().is_err() {
        println!("file already exists");
    };

    let mut data: Data = decerialize_file();
    println!("{data:?}");
}
