use anyhow::Context;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Endpoint {
    pub addr: Arc<str>,
    pub port: u16,
}

impl FromStr for Endpoint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr, port) = s
            .rsplit_once(':')
            .context("Failed parsing endpoint. Must be in the format of addr:port")?;

        if addr.trim().is_empty() {
            return Err(anyhow::anyhow!("Endpoint address cannot be empty"));
        }

        Ok(Endpoint {
            addr: addr.trim().into(),
            port: port
                .parse()
                .with_context(|| format!("Parsing \"{port}\" as a number"))?,
        })
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.addr, self.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_works() {
        let endpoint: Endpoint = "hello:23".parse().unwrap();
        assert_eq!(
            endpoint,
            Endpoint {
                addr: String::from("hello").into(),
                port: 23
            }
        );

        assert!(matches!(Endpoint::from_str(":123"), Result::Err(_)))
    }
}
