use crate::parse;

use std::{
    fmt,
    net::{AddrParseError, Ipv4Addr},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum RecordType {
    A,
    MX,
    CNAME,
    ALIAS,
    TXT,
    NS,
    AAAA,
    SRV,
    TLSA,
    CAA,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RecordType::*;
        write!(
            f,
            "{}",
            match self {
                A => "A",
                MX => "MX",
                CNAME => "CNAME",
                ALIAS => "ALIAS",
                TXT => "TXT",
                NS => "NS",
                AAAA => "AAAA",
                SRV => "SRV",
                TLSA => "TLSA",
                CAA => "CAA",
            },
        )
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Record {
    #[serde(deserialize_with = "parse::strnum")]
    pub id: u64,
    pub name: String,
    #[serde(deserialize_with = "parse::strnum")]
    pub ttl: u64,
    #[serde(rename = "type")]
    pub typ: RecordType,
    pub content: String,
    #[serde(deserialize_with = "parse::opt_strnum")]
    pub prio: Option<u64>,
    pub notes: Option<String>,
}

impl Record {
    pub fn ip(&self) -> Result<Ipv4Addr, AddrParseError> {
        self.content.parse()
    }
}

#[derive(Serialize)]
pub(crate) struct Req<'s> {
    pub name: Option<&'s str>,
    #[serde(rename = "type")]
    pub typ: RecordType,
    pub content: &'s str,
    pub ttl: Option<u64>,
    pub prio: Option<u64>,
}

pub(crate) fn fmt_dst(
    path: &str,
    domain: &str,
    typ: RecordType,
    subdomain: Option<&str>,
) -> String {
    if let Some(s) = subdomain {
        format!("{path}/{domain}/{typ}/{s}")
    } else {
        format!("{path}/{domain}/{typ}")
    }
}
