use common::{event, types};
use common::types::DataEventType;
use stream::dataflow;
use crate::constants;
use crate::actor::execution::Node;

pub struct InternalSink(types::JobID, Vec<actix::Addr<Node>>);

impl dataflow::Sink<Vec<event::DataEvent>> for InternalSink {
    fn sink(&self, output: Vec<event::DataEvent>) {
        if self.1.is_empty() {
            let client = data_client::new_data_engine_client(data_client::DataEngineConfig {
                host: None,
                port: None,
                uri: common::sysenv::get_env(constants::TABLEFLOW_URI_ENV_KEY),
            });
            let group = common::lists::group_hashmap(&output, |e| e.event_type.clone());
            group.iter().for_each(|(action, events)| {
                match action {
                    DataEventType::INSERT => {
                        let ref mut request = data_client::tableflow::InsertRequest::new();
                        let entries = common::lists::map(events, |e| types::Entry {
                            row_idx: e.row_idx,
                            value: e.data.clone(),
                        }.into());
                        request.set_headerId(self.0.header_id.clone());
                        request.set_tableId(self.0.table_id.clone());
                        request.set_entries(protobuf::RepeatedField::from(entries));
                        match client.insert(request) {
                            Err(err) => log::error!("insert {:?} failed. Details: {}", request, err),
                            _ => {}
                        }
                    }
                    DataEventType::UPDATE => {
                        let ref mut request = data_client::tableflow::UpdateRequest::new();
                        let entries = common::lists::map(events, |e| types::Entry {
                            row_idx: e.row_idx,
                            value: e.data.clone(),
                        }.into());
                        request.set_headerId(self.0.header_id.clone());
                        request.set_tableId(self.0.table_id.clone());
                        request.set_entries(protobuf::RepeatedField::from(entries));
                        match client.update(request) {
                            Err(err) => log::error!("update failed. Details: {}", err),
                            _ => {}
                        }
                    }
                    DataEventType::DELETE => {
                        let ref mut request = data_client::tableflow::DeleteRequest::new();
                        request.set_headerId(self.0.header_id.clone());
                        request.set_tableId(self.0.table_id.clone());
                        match client.delete(request) {
                            Err(err) => log::error!("delete failed. Details: {}", err),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            });
        }

        common::lists::for_each(&self.1, |addr| {
            addr.do_send(event::EventSet::new(output.clone()))
        })
    }
}
