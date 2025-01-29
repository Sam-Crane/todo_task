use crate::shared::Task;

use csv::Writer;
use serde_json::to_writer;
use printpdf::{BuiltinFont, PdfDocument, Mm};
use std::fs::File;
use std::io::{self, BufWriter};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct Exportable {
    pub tasks: Arc<Mutex<HashMap<usize, Task>>>,
}

impl Exportable {
    pub fn new(tasks: Arc<Mutex<HashMap<usize, Task>>>) -> Self {
        Exportable { tasks }
    }

    pub async fn export_to_csv(&self, filename: &str) -> io::Result<()> {
        let tasks = self.tasks.lock().await.clone();
        let mut wtr = Writer::from_path(filename)?;

        // Write headers
        wtr.write_record(&["ID", "Title", "Details", "Start Time", "End Time", "Recurring", "Frequency"])?;

        // Write tasks
        for task in tasks.values() {
            wtr.write_record(&[
                task.id.to_string(),
                task.title.clone(),
                task.details.clone(),
                task.start_time.to_string(),
                task.end_time.to_string(),
                task.is_recurring.to_string(),
                task.frequency_minutes.map_or("".to_string(), |f| f.to_string()),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub async fn export_to_json(&self, filename: &str) -> io::Result<()> {
        let tasks = self.tasks.lock().await.clone();
        let file = File::create(filename)?;
        to_writer(file, &tasks)?;
        Ok(())
    }

    pub async fn export_to_pdf(&self, filename: &str) -> io::Result<()> {
        let tasks = self.tasks.lock().await.clone();
        let (doc, page1, layer1) = PdfDocument::new("Todo Tasks", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("PDF Font Error: {:?}", e)))?;

        let mut current_layer = doc.get_page(page1).get_layer(layer1);
        let mut y_position = Mm(280.0);

        for task in tasks.values() {
            current_layer.use_text(
                format!(
                    "ID: {}, Title: {}, Details: {}, Start: {}, End: {}, Recurring: {}, Frequency: {:?}",
                    task.id,
                    task.title,
                    task.details,
                    task.start_time,
                    task.end_time,
                    if task.is_recurring { "Yes" } else { "No" },
                    task.frequency_minutes
                ),
                12.0,
                Mm(10.0),
                y_position,
                &font,
            );
            y_position -= Mm(10.0);

            if y_position < Mm(10.0) {
                // Add a new page if content exceeds one page
                let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "New Layer");
                y_position = Mm(280.0);
                current_layer = doc.get_page(new_page).get_layer(new_layer);
            }
        }

        let mut buffer = BufWriter::new(File::create(filename)?);
        doc.save(&mut buffer).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("PDF Save Error: {:?}", e)))?;

        Ok(())
    }
}
