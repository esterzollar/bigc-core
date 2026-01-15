use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Proxy;
use scraper::{Html, Selector};
use std::collections::HashMap;

#[derive(Clone)]
pub struct BigNet {
    client: Option<Client>,
    proxy: Option<String>,
    user_agent: Option<String>,
    headers: HashMap<String, String>,
}

impl BigNet {
    pub fn new() -> Self {
        BigNet {
            client: None,
            proxy: None,
            user_agent: None,
            headers: HashMap::new(),
        }
    }

    pub fn set_proxy(&mut self, url: &str) {
        self.proxy = Some(url.to_string());
        self.build_client(); // Rebuild client with new settings
    }

    pub fn set_user_agent(&mut self, agent: &str) {
        self.user_agent = Some(agent.to_string());
        self.build_client();
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
        self.build_client();
    }

    fn build_client(&mut self) {
        let mut builder = Client::builder();

        if let Some(p) = &self.proxy {
            if let Ok(proxy) = Proxy::all(p) {
                builder = builder.proxy(proxy);
            } else {
                println!("BigNet Error: Invalid Proxy URL '{}'", p);
            }
        }

        if let Some(ua) = &self.user_agent {
            builder = builder.user_agent(ua);
        }

        // Headers
        let mut header_map = HeaderMap::new();
        for (k, v) in &self.headers {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                header_map.insert(hn, hv);
            }
        }
        builder = builder
            .default_headers(header_map)
            .timeout(std::time::Duration::from_secs(30));

        match builder.build() {
            Ok(c) => self.client = Some(c),
            Err(e) => println!("BigNet Error: Failed to build client. {}", e),
        }
    }

    pub fn get(&mut self, url: &str) -> String {
        if self.client.is_none() {
            self.build_client();
        }

        if let Some(c) = &self.client {
            match c.get(url).send() {
                Ok(resp) => match resp.text() {
                    Ok(text) => text,
                    Err(e) => format!("BigNet Error: Bad Response Text. {}", e),
                },
                Err(e) => format!("BigNet Error: Request Failed. {}", e),
            }
        } else {
            String::from("BigNet Error: No Client")
        }
    }

    pub fn post(&mut self, url: &str, data: &str) -> String {
        if self.client.is_none() {
            self.build_client();
        }

        if let Some(c) = &self.client {
            let mut req = c.post(url);

            // Only add default form type if user hasn't specified a content type
            let has_ct = self
                .headers
                .keys()
                .any(|k| k.to_lowercase() == "content-type");
            if !has_ct {
                req = req.header("Content-Type", "application/x-www-form-urlencoded");
            }

            match req.body(data.to_string()).send() {
                Ok(resp) => match resp.text() {
                    Ok(text) => text,
                    Err(e) => format!("BigNet Error: Bad Response Text. {}", e),
                },
                Err(e) => format!("BigNet Error: Request Failed. {}", e),
            }
        } else {
            String::from("BigNet Error: No Client")
        }
    }

    pub fn look_for(&self, pattern: &str, html: &str) -> String {
        let document = Html::parse_document(html);
        if let Ok(selector) = Selector::parse(pattern) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(val) = element.value().attr("value") {
                    return val.to_string();
                } else {
                    return element.text().collect::<Vec<_>>().join("");
                }
            }
        }
        String::from("")
    }

    pub fn look_at_json(&self, key: &str, json: &str) -> String {
        let trimmed = json.trim();
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(val) => {
                if let Some(v) = val.get(key) {
                    if let Some(s) = v.as_str() {
                        return s.to_string();
                    }
                    return v.to_string();
                }
                String::from("")
            }
            Err(e) => {
                println!("BigNet Error: JSON Parse Failed. {}", e);
                String::from("")
            }
        }
    }
}
