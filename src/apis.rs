use std::collections::HashMap;
use crate::types::{Response, Request};

type Api = Box<dyn Fn(Request) -> Response + Send + Sync + 'static>;

pub struct ApiRegister {
    apis: HashMap<String, Api>,
    middleware: Vec<()>,
}

impl ApiRegister {
    pub fn new() -> Self {
        Self {
            apis: HashMap::new(),
            middleware: Vec::new(),
        }
    }

    pub fn register_api(&mut self, path: &str, api: Api) {
        self.apis.insert(path.into(), api);
    }

    pub fn get_api(&self, path: &str) -> Option<&Api> {
        self.apis.get(path)
    }
}