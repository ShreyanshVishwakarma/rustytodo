use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::io;
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

fn serialize_file(data: &Data) {
    let buff = serde_json::to_string(data)
        .unwrap_or_else(|err| panic!("unable to serialize to a json : {err:?}"));
    println!("{}", buff);
    fs::write("user_data.txt", buff.as_str())
        .unwrap_or_else(|err| panic!("unable to write to file : {err:?}"));
}

fn add_task(tasks: &mut Vec<String>) {
    println!("Enter the task : ");
    let mut task = String::new();
    io::stdin()
        .read_line(&mut task)
        .expect("failed to readline");

    tasks.push(task.trim().to_string());

    println!("Tasks :\n");
    for (index, task) in tasks.iter().enumerate() {
        println!("{}. {}", index + 1, task);
    }
}

fn complete_task(tasks: &mut Vec<String>, completed: &mut Vec<String>) {
    println!("Tasks :\n");
    for (index, task) in tasks.iter().enumerate() {
        println!("{}. {}", index + 1, task);
    }
    println!("Enter the Index of completed task");
    let mut completed_task = String::new();
    io::stdin()
        .read_line(&mut completed_task)
        .expect("failed to readline");
    let mut completed_task: usize = completed_task.trim().parse().unwrap();
    completed_task = completed_task - 1;
    if completed_task < tasks.len() {
        let removed_element = tasks.remove(completed_task);
        completed.push(removed_element);
    } else {
        println!("Index out of bounds!");
    }
    println!("Completed tasks :");
    for (index, task) in completed.iter().enumerate() {
        println!("{}. {}", index + 1, task);
    }
}

fn delete_task() {}

fn main() {
    if init().is_err() {
        println!("Welcome back");
    };

    let mut data: Data = decerialize_file();
    //println!("{data:?}");

    loop {
        println!(
            "TODO App:\n\n1. Add task\n2. Mark as complete\n3. Delete Task\n4. Exit\nEnter your choice:"
        );
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("failed to readline");
        match choice.trim() {
            "1" => add_task(&mut data.tasks),
            "2" => complete_task(&mut data.tasks, &mut data.completed),
            "3" => delete_task(),
            "4" => return, // Note: 'return' will exit the current function
            _ => println!("Enter a valid choice"),
        }
        serialize_file(&data);
    }
}
