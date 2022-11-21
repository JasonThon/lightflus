use crate::{
    apiserver::{create_resource_request::Options, CreateResourceRequest},
    common::Dataflow,
};

impl CreateResourceRequest {
    pub fn get_dataflow(&self) -> Dataflow {
        match &self.options {
            Some(options) => match options {
                Options::Dataflow(dataflow) => dataflow
                    .dataflow
                    .as_ref()
                    .map(|df| df.clone())
                    .unwrap_or_default(),
            },
            None => Default::default(),
        }
    }

    pub fn is_dataflow_empty(&self) -> bool {
        self.options.is_none()
            || self
                .options
                .as_ref()
                .filter(|options| match options {
                    Options::Dataflow(dataflow) => dataflow.dataflow.is_none(),
                })
                .is_some()
    }
}
