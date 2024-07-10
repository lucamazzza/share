use serde::{Deserialize, Serialize};

const HISTORY_SIZE: usize = 32;

#[derive(Serialize, Deserialize, Debug)]
pub struct History<T> {
    data: [Option<T>; HISTORY_SIZE],
    pointer: usize,
}

impl<T> History<T> {
    pub fn new() -> Self {
        let mut data = vec![];
        for _ in 0..HISTORY_SIZE {
            data.push(None);
        }
        Self {
            data: data
                .try_into()
                .unwrap_or_else(|_| panic!("Failed to generate history")),
            pointer: 0,
        }
    }
    
    pub fn push(&mut self, item: T) {
        self.data[self.pointer] = Some(item);
        self.pointer = (self.pointer + 1) % HISTORY_SIZE;
    } 
    
    pub fn get(&mut self, index: usize) -> Option<T> {
        self.data[index].as_ref()
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data[index].as_mut()
    }
    
    pub fn get_all(&self) -> &Vec<T> {
        &self.data.as_ref().clone().strip_suffix(&[None])
    }
    
    pub fn get_count(&self) -> usize {
        self.pointer
    }
}