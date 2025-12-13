//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: netsvc
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Implementacja serwera HTTP (szkielet).

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Head,
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: [u8; 256],
    pub path_len: usize,
}

impl HttpRequest {
    pub const fn new() -> Self {
        Self {
            method: HttpMethod::Get,
            path: [0; 256],
            path_len: 0,
        }
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 14 {
            return None;
        }

        let method = if buf.starts_with(b"GET ") {
            HttpMethod::Get
        } else if buf.starts_with(b"POST ") {
            HttpMethod::Post
        } else if buf.starts_with(b"HEAD ") {
            HttpMethod::Head
        } else {
            return None;
        };

        let start = if method == HttpMethod::Post { 5 } else { 4 };
        let mut end = start;
        while end < buf.len() && buf[end] != b' ' {
            end += 1;
        }

        let path_len = (end - start).min(256);
        let mut request = Self::new();
        request.method = method;
        request.path[..path_len].copy_from_slice(&buf[start..start + path_len]);
        request.path_len = path_len;

        Some(request)
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub body: [u8; 1024],
    pub body_len: usize,
}

impl HttpResponse {
    pub const fn new(status: u16) -> Self {
        Self {
            status,
            body: [0; 1024],
            body_len: 0,
        }
    }

    pub fn set_body(&mut self, body: &[u8]) {
        let len = body.len().min(1024);
        self.body[..len].copy_from_slice(&body[..len]);
        self.body_len = len;
    }
}
