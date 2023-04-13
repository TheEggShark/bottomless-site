use std::collections::HashMap;

type Api = Box<dyn Fn() -> () + Send + Sync + 'static>;

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

    pub fn run_api(&self, path: &str) {
        let api = self.apis.get(path).unwrap();
        api()
    }

    pub fn get_api(&self, path: &str) -> Option<&Api> {
        self.apis.get(path)
    }
}