use std::sync::Arc;

use crate::config::Database;

pub struct F123Cache {
    #[allow(unused)]
    db: Arc<Database>,
}

impl F123Cache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }
}
