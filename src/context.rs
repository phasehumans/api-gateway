use bytes::Bytes;
use http::{HeaderMap, Method, Uri};
use std::{
    collections::HashMap,
    net::IpAddr,
    time::Instant,
};

use crate::config::RouteConfig;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub client_ip: Option<IpAddr>,
    pub started_at: Instant,
    pub route: Option<RouteConfig>,
    pub chosen_upstream: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl RequestContext {
    pub fn new(
        request_id: String,
        method: Method,
        uri: Uri,
        headers: HeaderMap,
        body: Bytes,
        client_ip: Option<IpAddr>,
    ) -> Self {
        Self {
            request_id,
            method,
            uri,
            headers,
            body,
            client_ip,
            started_at: Instant::now(),
            route: None,
            chosen_upstream: None,
            metadata: HashMap::new(),
        }
    }
}
