use crate::shared::Task;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct ReadWrite {
    pub tasks: Arc<Mutex<HashMap<usize, Task>>>,
}

impl ReadWrite {
    pub async fn save_to_file(&self, folder_path: &str) -> io::Result<()> {
        let tasks = self.tasks.lock().await;

        for task in tasks.values() {
            let category_path = format!("{}/{}", folder_path, task.title);
            fs::create_dir_all(&category_path)?;

            let filename = format!("{}/task_{}.txt", category_path, task.id);
            let mut file = fs::File::create(&filename)?;

            writeln!(
                file,
                "- ID: {}\n  Title: {}\n  Details: {}\n  Start: {}\n  End: {}\n  Recurring: {}\n  Frequency: {:?}",
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

    pub async fn load_from_file(&self, folder_path: &str) -> io::Result<()> {
        let mut tasks = self.tasks.lock().await;
        tasks.clear();

        for entry in fs::read_dir(folder_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                for file in fs::read_dir(entry.path())? {
                    let file = file?;
                    let content = fs::read_to_string(file.path())?;

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
                            current_task.id = line.split(':').nth(1).unwrap().trim().parse().unwrap();
                        } else if line.starts_with("  Title:") {
                            current_task.title = line.split(':').nth(1).unwrap().trim().to_string();
                        } else if line.starts_with("  Details:") {
                            current_task.details = line.split(':').nth(1).unwrap().trim().to_string();
                        } else if line.starts_with("  Start:") {
                            current_task.start_time = line.split(':').nth(1).unwrap().trim().parse().unwrap();
                        } else if line.starts_with("  End:") {
                            current_task.end_time = line.split(':').nth(1).unwrap().trim().parse().unwrap();
                        } else if line.starts_with("  Recurring:") {
                            current_task.is_recurring = line.split(':').nth(1).unwrap().trim().parse().unwrap();
                        } else if line.starts_with("  Frequency:") {
                            current_task.frequency_minutes = line.split(':').nth(1).unwrap().trim().parse().ok();
                        }
                    }

                    tasks.insert(current_task.id, current_task);
                }
            }
        }

        Ok(())
    }
}
