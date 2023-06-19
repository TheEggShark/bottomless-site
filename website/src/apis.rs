use std::fmt::Debug;
use std::{collections::HashMap, time::Instant, net::IpAddr};
use std::sync::RwLock;
use crate::types::{Response, Request};

type InnerApi = Box<dyn Fn(Request) -> Response + Send + Sync + 'static>;


pub struct Api {
    inner: InnerApi,
    limit_count: usize,
    seconds_till_refresh: u32,
}

impl Debug for Api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Api")
            .field("limit_count", &self.limit_count)
            .field("seconds_till_refresh", &self.seconds_till_refresh)
            .finish()
    }
}

impl Api {
    pub fn run(&self, req: Request) -> Response {
        (self.inner)(req)
    }

    fn get_limit_and_refresh(&self) -> (usize, u32) {
        (self.limit_count, self.seconds_till_refresh)
    }
}

#[derive(Debug)]
pub struct ApiRegister {
    apis: HashMap<String, Api>,
    users: RwLock<HashMap<IpAddr, User>>,
}

impl ApiRegister {
    pub fn new() -> Self {
        Self {
            apis: HashMap::new(),
            users: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_api(&mut self, path: &str, inner_api: InnerApi, limit: usize, refresh_timer: u32) {
        let api = Api {
            inner: inner_api,
            limit_count: limit,
            seconds_till_refresh: refresh_timer,
        };
        self.apis.insert(path.into(), api);
    }

    pub fn get_api(&self, path: &str) -> Option<&Api> {
        self.apis.get(path)
    }

    pub fn user_exists(&self, ip: &IpAddr) -> bool {
        let reader = self.users.read().unwrap();
        reader.contains_key(ip)
    }

    pub fn check_limit(&self, ip: &IpAddr, api_path: &str) -> bool {
        let mut writer = self.users.write().unwrap();
        writer.get_mut(ip).unwrap().check_limit(api_path)
    }

    pub fn add_request(&self, api_path: &str, user_ip: IpAddr) {
        let mut writer = self.users.write().unwrap();
        writer.get_mut(&user_ip).unwrap().add_request(api_path);
    }

    pub fn add_gloabal_request(&self, user_ip: IpAddr) {
        let mut writer = self.users.write().unwrap();
        writer.get_mut(&user_ip).unwrap().add_gloabal_request();
    }

    pub fn add_user(&self, user_ip: IpAddr) {
        let limits = self.apis.iter()
            .map(|(k, v)| {
                let (limit, refresh) = v.get_limit_and_refresh();
                (limit,refresh, k.as_str())
            })
            .map(|tup| (RateLimiter::new(tup.0, tup.1), tup.2))
            .collect::<Vec<(RateLimiter, &str)>>();

        let mut user = User::new();
        user.add_many(limits);
        let mut inserter = self.users.write().unwrap();
        inserter.insert(user_ip, user);
    }

    pub fn clean_recent_requests(&self) {
        let reader = self.users.read().unwrap();
        let keys_to_remove = reader.iter()
            .filter(|(_, user)| user.get_recent_request_count() == 0)
            .map(|(key, _)| *key)
            .collect::<Vec<IpAddr>>();

        drop(reader);
        let mut inserter = self.users.write().unwrap();

        for key in keys_to_remove {
            inserter.remove(&key);
        }
    }
}

#[derive(Debug)]
struct User {
    limits: HashMap<String, RateLimiter>,
}

impl User {
    pub fn new() -> Self {
        let golobal_limiter = RateLimiter::new(36, 360);
        let mut limits = HashMap::new();
        limits.insert("global".to_string(), golobal_limiter);
        Self {
            limits
        }
    }

    pub fn check_limit(&mut self, api_path: &str) -> bool {
        if !self.limits.get_mut("global").unwrap().check_limit() {
            return false;
        }

        match self.limits.get_mut(api_path) {
            None => true,
            Some(limiter) => limiter.check_limit()
        }
    }

    pub fn add_many(&mut self, api_limits: Vec<(RateLimiter, &str)>) {
        api_limits.into_iter()
            .for_each(|(value, key)| {
                self.limits.insert(key.to_string(), value);
            });
    }

    pub fn get_recent_request_count(&self) -> usize {
        self.limits.iter()
            .map(|(_, limiter)| limiter.get_recent_request_count())
            .sum()
    }

    pub fn add_gloabal_request(&mut self) {
        self.limits.get_mut("global").unwrap().add_request(Instant::now());
    }

    pub fn add_request(&mut self, api_path: &str) {
        self.limits.get_mut("global").unwrap().add_request(Instant::now());
        self.limits.get_mut(api_path).unwrap().add_request(Instant::now());
    }
}


// so check if in past 6 seconds there's greater than or equal to 6 requests
// if yes then no request allowed
// so I can spam 6 requests instantly
// but then I have to wait for 6 seconds
// to refill my buffer

#[derive(Debug)]
struct RateLimiter {
    last_requests: Vec<Instant>,
    lockdown_time: Option<Instant>, //store how long untill they can make more requests
    seconds_till_refresh: u32,
    limit_count: usize,
}

impl RateLimiter {
    pub fn new(limit: usize, seconds_till_refresh: u32) -> Self {
        Self {
            last_requests: Vec::with_capacity(limit),
            lockdown_time: None,
            seconds_till_refresh,
            limit_count: limit
        }
    }


    pub fn check_limit(&mut self) -> bool {
        match self.lockdown_time {
            Some(time) => {
                if time.elapsed().as_secs() > self.seconds_till_refresh as u64 {
                    self.lockdown_time = None;
                } else {
                    // on lockdown no more requets !
                    return false;
                }
            }
            None => {},
        }


        let current_time = Instant::now();
        let reqs = std::mem::take(&mut self.last_requests);

        // clears old requests
        self.last_requests = reqs.into_iter()
            .filter(|time| current_time.duration_since(*time).as_secs() < self.seconds_till_refresh as u64)
            .collect::<Vec<Instant>>();

        if self.last_requests.len() > self.limit_count {
            // request should not go through and be put in lockdown
            self.lockdown_time = Some(Instant::now());
            return false;
        }

        true
    }

    pub fn add_request(&mut self, time: Instant) {
        self.last_requests.push(time);
    }

    pub(crate) fn get_recent_request_count(&self) -> usize {
        self.last_requests.len()
    }
}