pub mod handler;

#[derive(Debug, Clone)]
pub enum IOEvent {
    Initialize,
    Send,
}