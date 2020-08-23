// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::fmt::{Display, Formatter};
use core::str::FromStr;
use url::Url;

use lnpbp::bitcoin;
use lnpbp::bitcoin::Address;
use lnpbp::bp::blind::OutpointHash;
use lnpbp::rgb::{Bech32, ContractId, ToBech32};

#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum Error {
    #[derive_from]
    Url(::url::ParseError),

    WrongUrlScheme,

    NonNullAuthority,

    NoAsset,

    WrongAssetEncoding,

    NoAmount,

    WrongAmountEncoding,

    WrongOutpoint,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OutpointDescriptor {
    Utxo(bitcoin::OutPoint),
    Address(bitcoin::Address),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outpoint {
    BlindedUtxo(OutpointHash),
    Address(bitcoin::Address),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Invoice {
    pub contract_id: ContractId,
    pub outpoint: Outpoint,
    pub amount: f32,
}

impl From<OutpointDescriptor> for Outpoint {
    #[inline]
    fn from(descriptor: OutpointDescriptor) -> Self {
        match descriptor {
            OutpointDescriptor::Utxo(outpoint) => Self::BlindedUtxo(outpoint.into()),
            OutpointDescriptor::Address(addr) => Self::Address(addr),
        }
    }
}

impl FromStr for OutpointDescriptor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match (Address::from_str(s), bitcoin::OutPoint::from_str(s)) {
            (Ok(addr), _) => Ok(Self::Address(addr)),
            (_, Ok(outpoint)) => Ok(Self::Utxo(outpoint)),
            _ => Err(Error::WrongOutpoint),
        }
    }
}

impl FromStr for Outpoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match (Address::from_str(s), Bech32::from_str(s)) {
            (Ok(addr), _) => Ok(Self::Address(addr)),
            (_, Ok(Bech32::Outpoint(outpoint))) => Ok(Self::BlindedUtxo(outpoint)),
            _ => Err(Error::WrongOutpoint),
        }
    }
}

impl FromStr for Invoice {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        if url.scheme() != "rgb20" {
            return Err(Error::WrongUrlScheme);
        }
        if url.has_authority() {
            return Err(Error::NonNullAuthority);
        }
        let outpoint = url.path().parse()?;
        let (_, amount) = url
            .query_pairs()
            .find(|(x, _)| x == "amount")
            .ok_or(Error::NoAmount)?;
        let amount = amount.parse().map_err(|_| Error::WrongAmountEncoding)?;
        let (_, contract_id) = url
            .query_pairs()
            .find(|(x, _)| x == "asset")
            .ok_or(Error::NoAsset)?;
        let contract_id = contract_id.parse().map_err(|_| Error::WrongAssetEncoding)?;
        Ok(Invoice {
            contract_id,
            outpoint,
            amount,
        })
    }
}

impl Display for OutpointDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::Utxo(outpoint) => write!(f, "{}", outpoint),
            Self::Address(addr) => write!(f, "{}", addr),
        }
    }
}

impl Display for Outpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::BlindedUtxo(outpoint) => write!(f, "{}", Bech32::Outpoint(outpoint.clone())),
            Self::Address(addr) => write!(f, "{}", addr),
        }
    }
}

impl Display for Invoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> ::core::fmt::Result {
        let url = Url::parse(&format!(
            "rgb20:{}?asset={}&amount={}",
            self.outpoint,
            self.contract_id.to_bech32(),
            self.amount
        ))
        .expect("Internal Url generation error");
        write!(f, "{}", url)
    }
}
