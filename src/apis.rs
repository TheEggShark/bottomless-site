use std::{collections::HashMap, time::Instant, net::SocketAddr};
use crate::types::{Response, Request};

type Api = Box<dyn Fn(Request) -> Response + Send + Sync + 'static>;

pub struct ApiRegister {
    apis: HashMap<String, Api>,
    users: HashMap<SocketAddr, User>,
}

impl ApiRegister {
    pub fn new() -> Self {
        Self {
            apis: HashMap::new(),
            users: HashMap::new(),
        }
    }

    pub fn register_api(&mut self, path: &str, api: Api) {
        self.apis.insert(path.into(), api);
    }

    pub fn get_api(&self, path: &str) -> Option<&Api> {
        self.apis.get(path)
    }

    fn clean_recent_requests(&mut self) {
        let keys_to_remove = self.users.iter()
            .filter(|(_, user)| user.get_recent_request_count() == 0)
            .map(|(key, _)| *key)
            .collect::<Vec<SocketAddr>>();

        for key in keys_to_remove {
            self.users.remove(&key);
        }
    }
}

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

    pub fn get_recent_request_count(&self) -> usize {
        self.limits.iter()
            .map(|(_, limiter)| limiter.get_recent_request_count())
            .sum()
    }
}


// so check if in past 6 seconds there's greater than or equal to 6 requests
// if yes then no request allowed
// so I can spam 6 requests instantly
// but then I have to wait for 6 seconds
// to refill my buffer

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