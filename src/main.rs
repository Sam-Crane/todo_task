mod shared;
mod ex_csv;
mod read_write;
pub mod error;

use crate::shared::{AppState, Task, TaskUpdate};
use chrono::Utc;
use clap::{Args, Parser, Subcommand};
use std::sync::Arc;
use tokio::time::sleep;
use tokio::io;

#[derive(Parser)]
#[command(name = "Todo Task")]
#[command(about = "A CLI tool for task management")]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new Task
    Add(AddArgs), 
    /// List all tasks by title
    ListByTitle {
        /// Title of the tasks to list
        title: String,
    },
    /// List a task by ID
    ListByID {
        /// ID of the task to list
        id: usize,
    },
    /// Edit a task by its ID
    Edit(EditArgs),
    /// Save tasks to a file
    SaveToFile { 
        /// File name to save tasks
        filename: String 
    },
    /// Load tasks from a file
    LoadFromFile { 
        /// File name to load tasks
        filename: String 
    },
    /// Export tasks to CSV
    ExportToCSV {
        /// File name for the exported CSV
        filename: String,
    },
    /// Export tasks to JSON
    ExportToJSON {
        /// File name for the exported JSON
        filename: String,
    },
    /// Export tasks to PDF
    ExportToPDF {
        /// File name for the exported PDF
        filename: String,
    },
    /// Delete a task by its ID
    Delete {
        /// ID of the task to delete
        id: usize,
    },
}

#[derive(Args)]
struct AddArgs {
    /// Title of the task
    title: String,
    /// Details of the task
    details: String,
    /// Start time (ISO 8601 format, e.g., "2024-12-31T15:00:06")
    start_time: String,
    /// End time (ISO 8601 format, e.g., "2024-12-31T17:00:06")
    end_time: String,
    /// Whether the task is recurring
    #[arg(long)]
    recurring: bool,
    /// Frequency of recurrence in minutes (only for recurring tasks)
    #[arg(long, requires = "recurring")]
    frequency_minutes: Option<i64>,
}

#[derive(Args)]
struct EditArgs {
    /// ID of the task to edit
    id: usize,
    /// New title (optional)
    #[arg(long)]
    title: Option<String>,
    /// New details (optional)
    #[arg(long)]
    details: Option<String>,
    /// New start time (ISO 8601 format, optional)
    #[arg(long)]
    start_time: Option<String>,
    /// New end time (ISO 8601 format, optional)
    #[arg(long)]
    end_time: Option<String>,
    /// Set task as recurring (optional)
    #[arg(long)]
    recurring: Option<bool>,
    /// Frequency of recurrence in minutes (optional)
    #[arg(long)]
    frequency_minutes: Option<i64>,
}

// Send reminder at 5 mins before start and 2 mins before end
async fn schedule_reminders(task: Task, state: Arc<AppState>) {
    let reminder_time_start = task.start_time - chrono::Duration::minutes(5);
    let reminder_time_end = task.end_time - chrono::Duration::minutes(2);
    let now = Utc::now();

    // wait until 5 mins before start time
    if reminder_time_start > now {
        if let Ok(duration) = reminder_time_start.signed_duration_since(now).to_std() {
            sleep(duration).await;
            println!("Reminder: '{}' starts in 5 minutes!", task.title);
        }
    }

    //wait until 2 mins before end time
    if reminder_time_end > now {
        if let Ok(duration) = reminder_time_end.signed_duration_since(now).to_std() {
            sleep(duration).await;
            println!("Reminder: '{}' ends in 2 minutes!", task.title);
        }
    }

    // cloning the title field to reuse it after move
    
    let task_title = task.title.clone(); // clone the title
    // mark task as completed
    println!("Task '{}' is complete", task_title);

    // if the task is a recurring, schedule the next instance
    if task.is_recurring {
        if let Some(frequency) = task.frequency_minutes {
            let next_task = Task {
                id: 0,
                title: task.title.clone(),
                details: task.details.clone(),
                start_time: task.start_time + chrono::Duration::minutes(frequency),
                end_time: task.end_time + chrono::Duration::minutes(frequency),
                is_recurring: true,
                frequency_minutes: Some(frequency),
            };

            // Add the next task to the state
            let task_id = state.add_task(next_task.clone()).await;
            println!("Next recurring task scheduled with ID: {:?}", task_id);
 
            // Spawn a task to schedule the next reminder
            let state_clone = Arc::clone(&state);

            tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(async move {
                    schedule_reminders(next_task, state_clone).await;
                });
            }); 
        }
    }
}



// Main Application ENtry
#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let done_folder = "tasks_done".to_string();
    let state = Arc::new(AppState::new(done_folder));
    let cli = CLI::parse();

    match cli.command {
        Commands::ExportToCSV { filename } => {
            state.export_to_csv(&filename)
                .await
                .expect("Failed to export tasks to CSV");
        }
        Commands::ExportToJSON { filename } => {
            state.export_to_json(&filename)
                .await
                .expect("Failed to export tasks to JSON");
        }
        Commands::ExportToPDF { filename } => {
            state.export_to_pdf(&filename)
                .await
                .expect("Failed to export tasks to PDF"); 
        }
        Commands::SaveToFile { filename } => {
            state.save_to_file(&filename).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        Commands::LoadFromFile { filename } => {
            state.load_from_file(&filename).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        Commands::Add(args) => {
            let start_time = chrono::DateTime::parse_from_rfc3339(&args.start_time)
                .expect("Invalid start time format")
                .with_timezone(&Utc);
            let end_time = chrono::DateTime::parse_from_rfc3339(&args.end_time)
                .expect("Invalid end time format")
                .with_timezone(&Utc);

            if start_time <= Utc::now() {
                eprintln!("Error: Start time must be in the future.");
                return Ok(());
            }
            if end_time <= start_time {
                eprintln!("Error: End time must be after the start time.");
                return Ok(());
            }

            let task = Task::new(
                args.title,
                args.details,
                start_time,
                end_time,
                args.recurring,
                args.frequency_minutes,
            );

            let task_id = state.add_task(task.clone()).await;
            println!("Task '{}' added with ID: {:?}", task.title, task_id);

            tokio::spawn(schedule_reminders(task, Arc::clone(&state)));
            
        }
        Commands::ListByTitle { title } => {
            let tasks = state.list_tasks_by_title(&title).await;
            if tasks.is_empty() {
                println!("No tasks available to display.");
            } else {
                for task in tasks {
                    println!("{:?}", task);
                }
            }
        }
        Commands::ListByID { id } => {
            if let Some(task) = state.list_tasks_by_id(id).await {
                println!("{:?}", task);
            } else {
                println!("Task with ID {} not found.", id);
            }
        }
        Commands::Delete { id } => {
            match state.delete_task(id).await {
                Ok(_) => println!("Task {} deleted successfully.", id),
                Err(err) => eprintln!("Error deleting task {}: {}", id, err),
            }
        }
        Commands::Edit(EditArgs {
            id,
            title,
            details,
            start_time,
            end_time,
            recurring,
            frequency_minutes,
        }) => {
            let parsed_start_time = if let Some(time_str) = start_time {
                Some(chrono::DateTime::parse_from_rfc3339(&time_str)
                    .expect("Invalid start time format")
                    .with_timezone(&chrono::Utc))
            } else {
                None
            };

            let parsed_end_time = if let Some(time_str) = end_time {
                Some(chrono::DateTime::parse_from_rfc3339(&time_str)
                    .expect("Invalid end time format")
                    .with_timezone(&chrono::Utc))
            } else {
                None
            };

            let task_update = TaskUpdate {
                title,
                details,
                start_time: parsed_start_time,
                end_time: parsed_end_time,
                is_recurring: recurring,
                frequency_minutes,
            };

            match state.edit_task(id, task_update).await {
                Ok(_) => println!("Task {} updated successfully.", id),
                Err(err) => eprintln!("Error updating task {}: {}", id, err),
            }
        }
    }
    Ok(())
}