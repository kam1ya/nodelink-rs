//! Internal networking layer (REST client and WebSocket connection loop).

pub(crate) mod connection;
pub(crate) mod rest;

pub(crate) use connection::Connection;
pub(crate) use rest::RestClient;
