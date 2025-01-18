use std::fs;
use std::io::{self, Write};
use tokio::sync::Mutex;
use std::collections::HashMap;
use chrono::Utc;
use crate::shared::Task;

pub struct ReadWrite {
    pub tasks: Mutex<HashMap<u32, Task>>,
}

impl ReadWrite {
    pub async fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let tasks = self.tasks.lock().await;
        let mut file = fs::File::create(filename)?;
        for task in tasks.values() {
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

    pub async fn load_from_file(&self, filename: &str) -> io::Result<()> {
        let content = fs::read_to_string(filename)?;
        let mut tasks = self.tasks.lock().await;
        let mut next_id = 1;

        let mut current_task = Task {
            id: 0,
            title: String::new(),
            details: String::new(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            is_recurring: false,
            frequency_minutes: None,
        };

        for line in content.lines() {
            if line.starts_with("- ID:") {
                if current_task.id != 0 {
                    tasks.insert(current_task.id, current_task.clone());
                }
                let id: u32 = line.split(": ").nth(1).unwrap().parse().unwrap();
                next_id = next_id.max(id + 1);
                current_task = Task {
                    id,
                    title: String::new(),
                    details: String::new(),
                    start_time: Utc::now(),
                    end_time: Utc::now(),
                    is_recurring: false,
                    frequency_minutes: None,
                };
            } else if line.starts_with("  Title:") {
                current_task.title = line.split(": ").nth(1).unwrap().to_string();
            } else if line.starts_with("  Details:") {
                current_task.details = line.split(": ").nth(1).unwrap().to_string();
            } else if line.starts_with("  Start:") {
                current_task.start_time = line.split(": ").nth(1).unwrap().parse().unwrap();
            } else if line.starts_with("  End:") {
                current_task.end_time = line.split(": ").nth(1).unwrap().parse().unwrap();
            } else if line.starts_with("  Recurring:") {
                current_task.is_recurring = line.split(": ").nth(1).unwrap().parse().unwrap();
            } else if line.starts_with("  Frequency:") {
                current_task.frequency_minutes = line.split(": ").nth(1).unwrap().parse().ok();
            }
        }

        if current_task.id != 0 {
            tasks.insert(current_task.id, current_task);
        }

        Ok(())
    }
}
