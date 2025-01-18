mod shared;
mod ex_csv;
mod read_write;

use crate::ex_csv::Exportable;
use crate::shared::AppState;
use crate::read_write::ReadWrite;

use std::io;
use shared::Task;
use std::sync::{atomic::{AtomicU32, Ordering}, Arc};
use std::collections::HashMap;
use chrono::Utc;
use tokio::time::sleep;
use tokio::sync::Mutex;
use clap::{Args, Parser, Subcommand};


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
    /// List all tasks
    List,
    /// Remove a task by its ID
    Remove(RemoveArgs),
    /// Edit a task by its ID
    Edit(EditArgs),
    /// Save tasks to a file
    SaveToFile { 
        /// File name to save tasks
        filename: String 
    },
    /// Load tasks from a file
    LoadFromFile {
        /// File name to load tasks from
        filename: String,
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
    /// Move a task to the "done" folder
    MoveToDone {
        /// ID of the task
        id: u32,
        /// Name of the done folder
        done_folder: String,
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
struct RemoveArgs {
    /// ID of the task to be removed
    id: u32,
}

#[derive(Args)]
struct EditArgs {
    /// ID of the task to edit
    id: u32,
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

// Implementation block for AppState struct
impl AppState {
    pub fn new() -> Self {
        AppState {
            read_write: ReadWrite {
                tasks: Mutex::new(HashMap::new()),
            },
            tasks: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicU32::new(1), // Start IDs from 1
        }
    }
    
    // intialize a add task to the state
    pub async fn add_task(&self, task: Task) -> u32 {
        let mut tasks = self.tasks.lock().await;
        let task_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        tasks.insert(task_id, task);
        task_id
    }

    pub async fn list_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().await;
        let task_list = tasks.values().cloned().collect::<Vec<Task>>();
        if task_list.is_empty() {
            println!("No tasks found.");
        } else {
            for task in &task_list {
                println!(
                    "ID: {}, Title: '{}', Details: '{}', Start: {}, End: {}, Recurring: {}",
                    task.id,
                    task.title,
                    task.details,
                    task.start_time,
                    task.end_time,
                    if task.is_recurring { "Yes" } else { "No" }
                );
            }
        }
        task_list
    }

    // Adding the edit task method
    pub async fn edit_task(
        &self,
        task_id: u32,
        new_title: Option<String>,
        new_details: Option<String>,
        new_start_time: Option<chrono::DateTime<chrono::Utc>>,
        new_end_time: Option<chrono::DateTime<chrono::Utc>>,
        new_recurring: Option<bool>,
        new_frequency_minutes: Option<i64>,
    ) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            if let Some(title) = new_title {
                task.title = title;
            }
            if let Some(details) = new_details {
                task.details = details;
            }
            if let Some(start_time) = new_start_time {
                if start_time <= chrono::Utc::now() {
                    return Err("Start time must be in the future.".to_string());
                }
                task.start_time = start_time;
            }
            if let Some(end_time) = new_end_time {
                if end_time <= task.start_time {
                    return Err("End time must be after the start time.".to_string());
                }
                task.end_time = end_time;
            }
            if let Some(recurring) = new_recurring {
                task.is_recurring = recurring;
            }
            if let Some(frequency) = new_frequency_minutes {
                if !task.is_recurring {
                    return Err("Cannot set frequency for a non-recurring task.".to_string());
                }
                task.frequency_minutes = Some(frequency);
            }

            Ok(())
        } else {
            Err(format!("Task with ID {} not found.", task_id))
        }
    }

    // Adding the export to csv method
    pub async fn export_to_csv(&self, filename: &str) -> io::Result<()> {
        let exportable = Exportable ::new(Arc::clone(&self.tasks));
        exportable.export_to_csv(filename).await
    }


    // Adding the export to pdf method
    pub async fn export_to_pdf(&self, filename: &str) -> io::Result<()> {
        let exportable = Exportable ::new(Arc::clone(&self.tasks));
        exportable.export_to_pdf(filename).await
    }
    
    // Adding the export to json method
    pub async fn export_to_json(&self, filename: &str) -> io::Result<()> {
        let exportable = Exportable ::new(Arc::clone(&self.tasks));
        exportable.export_to_json(filename).await
    }

    // Adding the remove task method
    pub async fn remove_task(&self, task_id: u32) -> Option<Task> {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(&task_id)
    }
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

            // Schedule the next task after the frequency duration
            let delay_until_next_task = next_task.start_time - Utc::now();
            if let Ok(duration) = delay_until_next_task.to_std() {
                sleep(duration).await; // Wait until the next task's start time
            }

            // Add the next task to the state
            let task_id = state.add_task(next_task.clone()).await;
            println!("Next recurring task scheduled with ID: {}", task_id);
 
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



//Main Application ENtry
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());
    let cli = CLI::parse();

    match cli.command {
        Commands::ExportToCSV { filename } => {
            state.export_to_csv(&filename)
            .await.expect("Failed to export tasks to CSV");
        }
        Commands::ExportToJSON { filename } => {
            state.export_to_json(&filename)
            .await.expect("Failed to export tasks to JSON");
        }
        Commands::ExportToPDF { filename } => {
            state.export_to_pdf(&filename)
            .await.expect("Failed to export tasks to PDF");
        }
        Commands::SaveToFile { filename } => {
            state.save_to_file(&filename)
            .await.expect("Failed to save tasks to file");
        }
        Commands::LoadFromFile { filename } => {
            state.load_from_file(&filename)
            .await.expect("Failed to load tasks from file");
        }
        Commands::MoveToDone { id, done_folder } => {
            AppState::move_to_done_folder(&state.tasks, id, &done_folder)
            .await.expect("Failed to move task to done folder");
        }
        Commands::Add(args) => {
            let start_time = chrono::DateTime::parse_from_rfc3339(&args.start_time)
                .expect("Invalid start time format")
                .with_timezone(&Utc);
            let end_time = chrono::DateTime::parse_from_rfc3339(&args.end_time)
                .expect("Invalid end time format")
                .with_timezone(&Utc);

            // Validate start and end time
            if start_time <= Utc::now() {
                eprintln!("Error: Start time must be in the future.");
                return;
            }
            if end_time <= start_time {
                eprintln!("Error: End time must be after the start time.");
                return;
            }

            // Create the task
            let task = Task::new(
                args.title,
                args.details,
                start_time,
                end_time,
                args.recurring,
                args.frequency_minutes,
            );

            // Add the task to the state and get the task_id
            let task_id = state.add_task(task.clone()).await;
            println!("Task '{}' added with ID: {}", task.title, task_id);

            tokio::spawn(schedule_reminders(task, Arc::clone(&state)));
        }

        Commands::List => {
            let tasks = state.list_tasks().await;
            if tasks.is_empty() {
                println!("No tasks available to display.");
            }
        }

        Commands::Remove (RemoveArgs{ id }) => {
            if let Some(removed_task) = state.remove_task(id).await {
                println!("Removed task: {:?}", removed_task);
            } else {
                println!("Task with ID {} not found.", id);
            }   
        }

        Commands::Edit (EditArgs {
            id,
            title,
            details,
            start_time,
            end_time,
            recurring,
            frequency_minutes}) => {
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
        
            match state
                .edit_task(
                    id,
                    title,
                    details,
                    parsed_start_time,
                    parsed_end_time,
                    recurring,
                    frequency_minutes,
                )
                .await
            {
                Ok(_) => println!("Task {} updated successfully.", id),
                Err(err) => eprintln!("Error updating task {}: {}", id, err),
            }
        }        
    }
}

