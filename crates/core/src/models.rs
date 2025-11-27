//! Domain models for the application

/// A basic entity model
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u64,
    pub name: String,
}

impl Entity {
    pub fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }
}
