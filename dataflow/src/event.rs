use std::time::SystemTime;

use crate::err;
use crate::runtime;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum TableAction {
    HeaderSubmit {
        data: Vec<String>,
        table_id: String,
        header_id: String,
    },
    FormulaUpdate {
        table_id: String,
        header_id: String,
        graph: runtime::formula::FormulaGraph,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TableEvent {
    action: TableAction,
    event_time: std::time::SystemTime,
}

impl TableEvent {
    pub fn action(&self) -> &TableAction {
        &self.action
    }
}

impl Event<String, TableAction> for TableEvent {
    fn event_time(&self) -> SystemTime {
        self.event_time.clone()
    }

    fn get_key(&self) -> String {
        todo!()
    }

    fn get_value(&self) -> TableAction {
        todo!()
    }
}


pub trait Event<K, V> {
    fn event_time(&self) -> std::time::SystemTime;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}