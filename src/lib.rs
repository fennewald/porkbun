use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;

mod api;
pub mod dns;
mod parse;

pub struct Client {
    api_key: String,
    secret_key: String,
}

impl Client {
    pub fn from_env() -> Result<Client, env::VarError> {
        Ok(Client {
            api_key: env::var("PORKBUN_API_KEY")?,
            secret_key: env::var("PORKBUN_SECRET_KEY")?,
        })
    }

    pub fn new(api_key: String, secret_key: String) -> Client {
        Client {
            api_key,
            secret_key,
        }
    }

    fn request<S: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        payload: &S,
    ) -> Result<R, Error> {
        #[derive(Serialize)]
        struct Req<'k, T> {
            #[serde(rename = "apikey")]
            api_key: &'k str,
            #[serde(rename = "secretapikey")]
            secret_key: &'k str,
            #[serde(flatten)]
            payload: T,
        }

        api::request(
            endpoint,
            &Req {
                api_key: &self.api_key,
                secret_key: &self.secret_key,
                payload,
            },
        )
    }

    fn request_rec_list(&self, path: &str) -> Result<Vec<dns::Record>, Error> {
        #[derive(Deserialize)]
        struct Res {
            records: Vec<dns::Record>,
        }

        self.request(path, &()).map(|r: Res| r.records)
    }

    pub fn ping(&self) -> Result<Ipv4Addr, Error> {
        #[derive(Debug, Deserialize)]
        struct Res {
            #[serde(rename = "yourIp")]
            your_ip: Ipv4Addr,
        }

        self.request("/ping", &()).map(|r: Res| r.your_ip)
    }

    pub fn update_ns(&self, domain: &str, nameservers: &[&str]) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Req<'s> {
            ns: &'s [&'s str],
        }

        self.request(
            &format!("/domain/updateNS/{domain}"),
            &Req { ns: nameservers },
        )
    }

    pub fn create_dns(
        &self,
        domain: &str,
        name: Option<&str>,
        typ: dns::RecordType,
        content: &str,
        ttl: Option<u64>,
        prio: Option<u64>,
    ) -> Result<u64, Error> {
        #[derive(Deserialize)]
        struct Res {
            id: u64,
        }

        self.request(
            &format!("/dns/create/{domain}"),
            &dns::Req {
                name,
                typ,
                content,
                ttl,
                prio,
            },
        )
        .map(|r: Res| r.id)
    }

    pub fn edit_dns(
        &self,
        domain: &str,
        id: u64,
        name: Option<&str>,
        typ: dns::RecordType,
        content: &str,
        ttl: Option<u64>,
        prio: Option<u64>,
    ) -> Result<(), Error> {
        self.request(
            &format!("/dns/edit/{domain}/{id}"),
            &dns::Req {
                name,
                typ,
                content,
                ttl,
                prio,
            },
        )
    }

    pub fn edit_dns_dst(
        &self,
        domain: &str,
        typ: dns::RecordType,
        subdomain: Option<&str>,
        content: &str,
        ttl: Option<u64>,
        prio: Option<u64>,
    ) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Req<'s> {
            content: &'s str,
            ttl: Option<u64>,
            prio: Option<u64>,
        }
        self.request(
            &dns::fmt_dst("/dns/editByNameType", domain, typ, subdomain),
            &Req { content, ttl, prio },
        )
    }

    pub fn delete_dns(&self, domain: &str, id: u64) -> Result<(), Error> {
        self.request(&format!("/dns/delete/{domain}/{id}"), &())
    }

    pub fn delete_dns_dst(
        &self,
        domain: &str,
        typ: dns::RecordType,
        subdomain: Option<&str>,
    ) -> Result<(), Error> {
        self.request(
            &dns::fmt_dst("/dns/deleteByNameType", domain, typ, subdomain),
            &(),
        )
    }

    pub fn retrieve_dns(&self, domain: &str, id: u64) -> Result<Vec<dns::Record>, Error> {
        self.request_rec_list(&format!("/dns/retrieve/{domain}/{id}"))
    }

    pub fn retrieve_dns_dst(
        &self,
        domain: &str,
        typ: dns::RecordType,
        subdomain: Option<&str>,
    ) -> Result<Vec<dns::Record>, Error> {
        self.request_rec_list(&dns::fmt_dst(
            "/dns/retrieveByNameType",
            domain,
            typ,
            subdomain,
        ))
    }

    pub fn list_dns(&self, domain: &str) -> Result<Vec<dns::Record>, Error> {
        self.request_rec_list(&format!("/dns/retrieve/{domain}"))
    }

    pub fn retrieve_ssl(&self, domain: &str) -> Result<SSLResponse, Error> {
        self.request(&format!("/ssl/retrieve/{domain}"), &())
    }
}

#[derive(Debug, Deserialize)]
pub struct SSLResponse {
    #[serde(rename = "intermediatecertificate")]
    pub intermediate_cert: String,
    #[serde(rename = "certificatechain")]
    pub certificate_chain: String,
    #[serde(rename = "publickey")]
    pub public_key: String,
    #[serde(rename = "privatekey")]
    pub private_key: String,
}

#[derive(Debug)]
pub enum Error {
    ApiError(String),
    RequestError(reqwest::Error),
    ParseError,
    /// Should be impossible. Implies either (1) an error in the program logic, or (2) porkbun's api changed
    UnexpectedError,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::RequestError(value)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
enum Status {
    Success,
    Error,
}

#[derive(Deserialize, Debug)]
pub struct Price {
    #[serde(deserialize_with = "parse::money")]
    pub registration: f64,
    #[serde(deserialize_with = "parse::money")]
    pub renewal: f64,
    #[serde(deserialize_with = "parse::money")]
    pub transfer: f64,
    #[serde(deserialize_with = "parse::coupon")]
    pub coupons: HashMap<String, Coupon>,
}

#[derive(Deserialize, Debug)]
pub struct Coupon {
    pub code: String,
    pub max_per_user: u64,
    #[serde(deserialize_with = "parse::yn")]
    pub first_year_only: bool,
    #[serde(rename = "type")]
    pub typ: String,
    pub amount: u64,
}

pub fn pricing() -> Result<HashMap<String, Price>, Error> {
    api::request("/pricing/get", &())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_prices() {
        dbg!(pricing().unwrap());
    }

    #[test]
    fn enumerate_dns() {
        let domain = "amyis.gay";
        let client = Client::from_env().unwrap();
        let records = client.list_dns(domain).unwrap();
        for rec in records {
            assert_eq!(
                rec,
                *client
                    .retrieve_dns(domain, rec.id)
                    .unwrap()
                    .first()
                    .unwrap()
            )
        }
    }
}
