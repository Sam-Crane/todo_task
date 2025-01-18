# 1. Introduction

## Todo CLI Tool

The **Todo CLI Tool** is a cross-platform, statically linked command-line application designed to help professionals manage tasks and reminders efficiently. With recurring task support, the tool is perfect for staying organized on Linux.

# 2. Usage Instructions

## Add a New Task
Add a task with specific start and end times, and optionally make it recurring:
```bash
todo_task add "Team Meeting" "Discuss project updates" "2024-12-31T15:00:00Z" "2024-12-31T16:00:00Z" 1440
```
View all Tasks and their details
```bash
todo_task list
```
Edit an existing task by providing the task ID and the new details:
```bash
todo_task edit <task_id> --title "Updated Title" --details "Updated Details" --start_time "2024-12-31T15:00:00Z" --end_time "2024-12-31T16:00:00Z" --recurring --frequency_minutes 1440
```
Remove a task by providing the task ID:
```bash
todo_task remove <task_id>
```
Export tasks to different formats:
```bash
todo_task export csv <filename>
```
```bash
todo_task export json <filename>
```
```bash
todo_task export pdf <filename>
```
Move a task to the done folder by providing the task ID and the done folder path:
```bash
todo_task done <task_id> <done_folder>
```
View all available commands and flags:
```bash
todo_task --help
```
#  3. Installation
## To install the Todo CLI Tool, clone the repository and build the project using Cargo:
```bash
git clone https://github.com/your_username/todo-cli-tools.git
cd todo-cli-tools
cargo build --release
```
After building the project, you can run the tool using:
```bash
./target/release/todo_task <command>
```