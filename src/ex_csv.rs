use crate::{error::TaskError, shared::Task};
use printpdf::{BuiltinFont, PdfDocument, Mm};
use std::{io::BufWriter, fs::File};

use csv::Writer;
use serde_json::to_writer;
use std::sync::Arc;
use dashmap::DashMap;
pub struct Exportable {
    pub tasks: Arc<DashMap<usize, Task>>,
}

impl Exportable {
    pub fn new(tasks: Arc<DashMap<usize, Task>>) -> Self {

        Exportable { tasks }
    }

    pub async fn export_to_csv(&self, filename: &str) -> Result<(), TaskError> {
        let tasks = self.tasks.iter();
        let mut wtr = Writer::from_path(filename)?;
        
        wtr.write_record(&["ID", "Title", "Details", "Start", "End", "Recurring", "Frequency"])?;
        
        for task in tasks {
            wtr.serialize(serde_json::to_value(task.value())?)?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub async fn export_to_json(&self, filename: &str) -> Result<(), TaskError> {
        let tasks: Vec<_> = self.tasks.iter().map(|entry| entry.value().clone()).collect();
        let file = File::create(filename)?;
        to_writer(file, &tasks)?;
        Ok(())
    }

    pub async fn export_to_pdf(&self, filename: &str) -> Result<(), TaskError> {
        let tasks = self.tasks.iter();
        let (doc, page, layer) = PdfDocument::new("Todo Tasks", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| TaskError::PdfError(e.to_string()))?;

        let mut current_layer = doc.get_page(page).get_layer(layer);
        let mut y_position = Mm(280.0);
        
        for task in tasks {
            let text = format!(
                "ID: {}\nTitle: {}\nDetails: {}\nStart: {}\nEnd: {}\nRecurring: {}\nFrequency: {}",
                task.id,
                task.title,
                task.details,
                task.start_time,
                task.end_time,
                task.is_recurring,
                task.frequency_minutes.unwrap_or(0)
            );
            
            current_layer.use_text(text, 12.0, Mm(10.0), y_position, &font);
            y_position -= Mm(15.0);
            
            if y_position < Mm(20.0) {
                let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "New Page");
                current_layer = doc.get_page(new_page).get_layer(new_layer);
                y_position = Mm(280.0);
            }
        }
        
        doc.save(&mut BufWriter::new(File::create(filename)?))
            .map_err(|e| TaskError::PdfError(e.to_string()))?;
        Ok(())
    }
}
