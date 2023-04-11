use std::collections::HashMap;

pub struct ApiRegister {
    apis: HashMap<String, ()>,
    middleware: Vec<()>,
}

impl ApiRegister {
    pub fn register_api(&mut self, path: &str, api: ()) {
        self.apis.insert(path.into(), api);
    }
}