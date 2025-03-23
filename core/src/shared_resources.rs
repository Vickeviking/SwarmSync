use crate::services::logger::Logger;
use std::sync::Arc;

pub struct SharedResources {
    pub logger: Arc<Logger>,
}

impl SharedResources {
    pub fn new(logger: Arc<Logger>) -> Self {
        SharedResources { logger }
    }
}
