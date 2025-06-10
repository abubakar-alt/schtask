#[macro_use]
extern crate litcrypt;

use_litcrypt!("ageofmachine");

use schtask::create_task;

fn main() {

    let result = create_task("MyTask", "C:\\Windows\\System32\\notepad.exe");
    println!("{}", result);
}