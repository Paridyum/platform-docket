//! Serving the operator interface over loopback, as JSON.
//!
//! This is an adapter in the sense STD-0004 means: everything here parses,
//! serialises or listens, and nothing here decides. Every route calls an
//! `application::` function that the CLI also calls, so the console and the
//! command line cannot disagree about what a person owes — see the design at
//! `docs/superpowers/specs/2026-08-21-served-hr-platform-design.md` §7.
//!
//! The HTTP implementation is our own, on `std::net`, rather than a crate. That
//! is a deliberate narrowing: the surface needed is one client over loopback with
//! no TLS and no keep-alive requirement, and the obvious dependency for the job
//! last published in 2022. See §0 of the design.

pub mod dto;
pub mod operator;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
