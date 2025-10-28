//! Governance contracts for The Manifold Web
//!
//! Implements:
//! - ManifoldGovernanceToken: Native governance token with voting power
//! - ManifoldDAO: Decentralized governance with proposals and voting

pub mod token;
pub mod dao;

pub use token::{GovernanceToken, TokenConfig};
pub use dao::{ManifoldDAO, DAOConfig, Proposal, ProposalStatus, Vote};
