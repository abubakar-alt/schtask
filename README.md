# Windows Task Scheduler Rust Implementation

This project is a Rust implementation of the Windows Task Scheduler functionality, specifically focusing on creating tasks that execute when a user logs on. It is based on the [Microsoft Windows Task Scheduler Logon Trigger Example](https://learn.microsoft.com/en-us/windows/win32/taskschd/logon-trigger-example--c---).

## Features

- Create scheduled tasks that run when a user logs on
- Dynamically find Task Scheduler GUIDs from the Windows Registry
- Support for custom executable paths
- Automatic cleanup of existing tasks with the same name
- Proper COM initialization and cleanup
- Error handling and reporting

## Dependencies

The project uses the following main dependencies:
- `winapi` - For Windows API bindings
- `winreg` - For Windows Registry access

## Usage

Add the following to your Cargo.toml

```rust
[dependencies]
noldr = { git = "https://github.com/Teach2Breach/schtask.git", branch = "main" }
```

Use in your project:

```rust
use schtask::create_task;

fn main() {
    // Create a task that runs notepad.exe when the user logs on
    let result = create_task("MyTask", "C:\\Windows\\System32\\notepad.exe");
    println!("{}", result);
}
```

## Implementation Details

The implementation follows the Windows Task Scheduler COM interface pattern:
1. Initialize COM and security settings
2. Create an instance of the Task Service
3. Get the root task folder
4. Create a new task definition
5. Configure task settings and registration info
6. Add a logon trigger
7. Set up the executable action
8. Register the task

## Security Considerations

- The task is created with the current user's credentials
- Tasks are created with interactive token logon type
- The implementation includes proper COM security initialization