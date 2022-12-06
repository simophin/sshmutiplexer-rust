use std::fmt::{Display, Formatter};
use std::str::FromStr;
use anyhow::{bail, Context};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Endpoint {
    pub addr: String,
    pub port: u16
}

impl FromStr for Endpoint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(':');
        match (splits.next(), splits.next(), splits.next()) {
            (Some(addr), Some(port), None) if !addr.trim().is_empty() => {
                Ok(Endpoint {
                    addr: addr.trim().to_string(),
                    port: port.parse().with_context(|| format!("Parsing \"{port}\" as a number"))?,
                })
            }
            _ => bail!("Invalid endpoint: '{s}'. Must be in the format of addr:port"),
        }
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
        assert_eq!(endpoint, Endpoint { addr: String::from("hello"), port: 23 });

        assert!(matches!(Endpoint::from_str(":123"), Result::Err(_)))
    }
}