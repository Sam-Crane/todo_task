use crate::shared::Task;
use dashmap::DashMap;
use crate::error::TaskError;


use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct ReadWrite {
    pub tasks: Arc<DashMap<usize, Task>>,
}


impl ReadWrite {
    pub async fn save_to_file(&self, folder_path: &str) -> Result<(), TaskError> {
        let tasks = self.tasks.clone();
        let mut file = fs::File::create(folder_path).await?;
        let tasks: Vec<_> = tasks.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect();
        let contents = serde_json::to_string(&tasks)?;
        file.write_all(contents.as_bytes()).await?;
        Ok(())
    }

    pub async fn load_from_file(&self, folder_path: &str) -> Result<(), TaskError> {
        let tasks = self.tasks.clone();
        let contents = fs::read_to_string(folder_path).await?;
        let deserialized_tasks: Vec<(usize, Task)> = serde_json::from_str(&contents)?;
        tasks.clear();
        for (key, value) in deserialized_tasks {
            tasks.insert(key, value);
        }
        Ok(())
    }
}