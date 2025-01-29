use crate::read_write::ReadWrite;
use crate::ex_csv::Exportable;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
use tokio::{sync::Mutex, time::sleep};
//use std::path::Path;
use std::time::Duration;

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

pub struct AppState {
    pub tasks: Arc<Mutex<HashMap<usize, Task>>>,
    pub next_id: Arc<AtomicUsize>,
    pub done_folder: String,
    pub exportable: Exportable,
    pub read_write: ReadWrite,
}

impl Task {
    pub fn new(
        title: String,
        details: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        is_recurring: bool,
        frequency_minutes: Option<i64>,
    ) -> Self {
        Task {
            id: 0,
            title,
            details,
            start_time,
            end_time,
            is_recurring,
            frequency_minutes,
        }
    }
}

pub struct TaskUpdate {
    pub title: Option<String>,
    pub details: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub is_recurring: Option<bool>,
    pub frequency_minutes: Option<i64>,
}

impl AppState {
    pub fn new(done_folder: String) -> Self {
        fs::create_dir_all(&done_folder).unwrap();
        let tasks = Arc::new(Mutex::new(HashMap::new()));
        AppState {
            read_write: ReadWrite { tasks: tasks.clone() },
            exportable: Exportable::new(Arc::clone(&tasks)),
            tasks: Arc::clone(&tasks),
            next_id: Arc::new(AtomicUsize::new(1)),
            done_folder,
        }
    }

    pub async fn add_task(&self, mut task: Task) -> usize {
        let mut tasks = self.tasks.lock().await;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        task.id = id;
        tasks.insert(id, task.clone());
        self.save_task_to_file(&task).await.unwrap();

        let tasks_clone = Arc::clone(&self.tasks);
        let done_folder_clone = self.done_folder.clone();
        let task_clone = task.clone();
        tokio::spawn(async move {
            AppState::monitor_task(tasks_clone, done_folder_clone, task_clone).await;
    });
        id
    }

    pub async fn save_task_to_file(&self, task: &Task) -> io::Result<()> {
        let category_path = format!("{}/{}", self.done_folder, task.title);
        fs::create_dir_all(&category_path)?;

        let filename = format!("{}/task_{}.txt", category_path, task.id);
        let mut file = fs::File::create(&filename)?;
        writeln!(
            file,
            "- ID: {}\n  Title: {}\n  Details: {}\n  Start: {}\n  End: {}\n  Recurring: {}\n  Frequency: {:?}",
            task.id, task.title, task.details, task.start_time, task.end_time, task.is_recurring, task.frequency_minutes
        )?;
        Ok(())
    }

    async fn monitor_task(tasks: Arc<Mutex<HashMap<usize, Task>>>, done_folder: String, task: Task) {
        let duration = (task.end_time - Utc::now()).to_std().unwrap_or(Duration::from_secs(0));
        sleep(duration).await;
        let mut tasks = tasks.lock().await;
        if tasks.remove(&task.id).is_some() {
            let category_path = format!("{}/{}", done_folder, task.title);
            fs::create_dir_all(&category_path).unwrap();

            let filename = format!("{}/task_{}.txt", category_path, task.id);
            let mut file = fs::File::create(&filename).unwrap();
            writeln!(
                file,
                "- ID: {}\n  Title: {}\n  Details: {}\n  Start: {}\n  End: {}\n  Recurring: {}\n  Frequency: {:?}",
                task.id, task.title, task.details, task.start_time, task.end_time, task.is_recurring, task.frequency_minutes
            ).unwrap();
            println!("Task {} moved to done folder.", task.id);
        }
    }

    pub async fn delete_task(&self, id: usize) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.remove(&id) {
            let category_path = format!("{}/{}", self.done_folder, task.title);
            let filename = format!("{}/task_{}.txt", category_path, task.id);
            if fs::remove_file(&filename).is_err() {
                eprintln!("Warning: Failed to delete file {}", filename);
            }
            Ok(())
        } else {
            Err(format!("Task with ID {} not found.", id))
        }
    }

    pub async fn list_tasks_by_title(&self, title: &str) -> Vec<Task> {
        let tasks = self.tasks.lock().await;
        tasks
            .values()
            .filter(|task| task.title.eq(title))
            .cloned()
            .collect()
    }

    pub async fn list_tasks_by_id(&self, id: usize) -> Option<Task> {
        let tasks = self.tasks.lock().await;
        tasks.get(&id).cloned()
    }

    pub async fn edit_task(&self, id: usize, update: TaskUpdate) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(&id) {
            if let Some(title) = update.title {
                task.title = title;
            }
            if let Some(details) = update.details {
                task.details = details;
            }
            if let Some(start_time) = update.start_time {
                task.start_time = start_time;
            }
            if let Some(end_time) = update.end_time {
                task.end_time = end_time;
            }
            if let Some(is_recurring) = update.is_recurring {
                task.is_recurring = is_recurring;
            }
            if let Some(frequency_minutes) = update.frequency_minutes {
                task.frequency_minutes = Some(frequency_minutes);
            }
            self.save_task_to_file(task).await.unwrap();
            Ok(())
        } else {
            Err(format!("Task with ID {} not found.", id))
        }
    }

    pub async fn export_to_csv(&self, filename: &str) -> io::Result<()> {
        self.exportable.export_to_csv(filename).await
    }

    pub async fn export_to_json(&self, filename: &str) -> io::Result<()> {
        self.exportable.export_to_json(filename).await
    }

    pub async fn export_to_pdf(&self, filename: &str) -> io::Result<()> {
        self.exportable.export_to_pdf(filename).await
    }
    pub async fn save_to_file(&self, folder_path: &str) -> io::Result<()> {
        self.read_write.save_to_file(folder_path).await
    }
    pub async fn load_from_file(&self, folder_path: &str) -> io::Result<()> {
        self.read_write.load_from_file(folder_path).await
    }
}