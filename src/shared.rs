use crate::read_write::ReadWrite;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicU32};
use tokio::sync::Mutex;
use std::fs;
use std::io::{self, Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: usize,
    pub title: String,
    pub details: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_recurring: bool,
    pub frequency_minutes: Option<i64>,
}

// Make Task explicitly Send + Sync
//unsafe impl Send for Task {}
//unsafe impl Sync for Task {}

pub struct AppState {
    pub read_write: ReadWrite,
    pub tasks: Arc<Mutex<HashMap<usize, Task>>>,
    pub next_id: AtomicU32,
}

impl Task {
    // Constructor method for Task to set default id to 0
    pub fn new(
        title: String, 
        details: String, 
        start_time: chrono::DateTime<Utc>, 
        end_time: chrono::DateTime<Utc>, 
        is_recurring: bool, 
        frequency_minutes: Option<i64>,
    ) -> Self {
        Task {
            id: 0,  // default id is 0
            title,
            details,
            start_time,
            end_time,
            is_recurring,
            frequency_minutes,
        }
    }
}

impl AppState {
    pub async fn move_to_done_folder(tasks: &Mutex<HashMap<usize, Task>>, task_id: usize, done_folder: &str) -> io::Result<()> {
        let mut tasks = tasks.lock().await;
        if let Some(task) = tasks.remove(&task_id) {
            let filename = format!("{}/task_{}.txt", done_folder, task_id);
            let mut file = fs::File::create(&filename)?;
            writeln!(
                file,
                "- ID: {}\n  Title: {}\n  Details: {}\n  Start: {}\n  End: {}\n  Recurring: {}\n  Frequency: {:?}\n",
                task.id,
                task.title,
                task.details,
                task.start_time,
                task.end_time,
                task.is_recurring,
                task.frequency_minutes
            )?;
        }
        Ok(())
    }

    pub async fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        self.read_write.save_to_file(file_path).await
    }

    pub async fn load_from_file(&self, file_path: &str) -> io::Result<()> {
        self.read_write.load_from_file(file_path).await
    }
}
