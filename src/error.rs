use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Time parsing error: {0}")]
    ChronoParse(#[from] chrono::ParseError),
    
    #[error("Task not found: {0}")]
    NotFound(usize),
    
    #[error("Invalid time: {0}")]
    InvalidTime(String),
    
    #[error("PDF generation error: {0}")]
    PdfError(String),
    
    #[error("CSV serialization error: {0}")]
    CsvError(#[from] csv::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}