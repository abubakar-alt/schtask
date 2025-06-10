use schtask::create_task;

fn main() {
    // Example 1: Create a task without arguments
    let result = create_task("MyTask", "C:\\Windows\\System32\\notepad.exe", None);
    println!("Task without arguments: {}", result);

    // Example 2: Create a task with arguments
    let result = create_task(
        "MyTaskWithArgs", 
        "C:\\Windows\\System32\\notepad.exe", 
        Some("C:\\Windows\\System32\\drivers\\etc\\hosts")
    );
    println!("Task with arguments: {}", result);
}