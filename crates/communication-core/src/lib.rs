pub mod app;
pub mod config;
pub mod domain;
pub mod transport;

pub use app::PolicyEngine;
pub use config::CommunicationConfig;
pub use domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DmTransportPolicy,
    PolicyContext, PolicyError, SendEnvelope, SessionProvenance, TransportProfile,
};

#[cfg(test)]
mod tests;
