use crate::modules::email_vault_db::{self, EmailAccount};
use base64::prelude::*;
use boring2::ssl::{SslConnector, SslMethod, SslStream, SslVerifyMode};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::*;

pub enum EmailStream {
    Plain(TcpStream),
    Tls(SslStream<TcpStream>),
}

impl Read for EmailStream {
    pub(crate) fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            EmailStream::Plain(s) => s.read(buf),
            EmailStream::Tls(s) => s.read(buf),
        }
    }
}

impl Write for EmailStream {
    pub(crate) fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            EmailStream::Plain(s) => s.write(buf),
            EmailStream::Tls(s) => s.write(buf),
        }
    }

    pub(crate) fn flush(&mut self) -> std::io::Result<()> {
        match self {
            EmailStream::Plain(s) => s.flush(),
            EmailStream::Tls(s) => s.flush(),
        }
    }
}

impl EmailStream {
    pub fn upgrade_to_tls(self, host: &str) -> Result<Self, String> {
        match self {
            EmailStream::Plain(tcp) => {
                let mut builder = SslConnector::builder(SslMethod::tls())
                    .map_err(|e| format!("Failed to create TLS connector: {}", e))?;
                builder.set_verify(SslVerifyMode::NONE);
                let connector = builder.build();
                let tls = connector
                    .configure()
                    .map_err(|e| format!("Failed to configure TLS: {}", e))?
                    .verify_hostname(false)
                    .connect(host, tcp)
                    .map_err(|e| format!("TLS handshake failed with '{}': {}", host, e))?;
                Ok(EmailStream::Tls(tls))
            }
            EmailStream::Tls(_) => Ok(self),
        }
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        match self {
            EmailStream::Plain(s) => s.set_read_timeout(timeout).map_err(|e| e.to_string()),
            EmailStream::Tls(s) => s
                .get_ref()
                .set_read_timeout(timeout)
                .map_err(|e| e.to_string()),
        }
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        match self {
            EmailStream::Plain(s) => s.set_write_timeout(timeout).map_err(|e| e.to_string()),
            EmailStream::Tls(s) => s
                .get_ref()
                .set_write_timeout(timeout)
                .map_err(|e| e.to_string()),
        }
    }
}

pub fn is_implicit_tls_smtp(port: u16, encryption_type: &str) -> bool {
    if port == 465 {
        return true;
    }
    let enc = encryption_type.trim();
    if enc.eq_ignore_ascii_case("SSL") {
        return true;
    }
    if enc.eq_ignore_ascii_case("SMTPS") {
        return true;
    }
    false
}

pub fn is_starttls_smtp(port: u16, encryption_type: &str) -> bool {
    if is_implicit_tls_smtp(port, encryption_type) {
        return false;
    }
    if port == 587 {
        return true;
    }
    let enc = encryption_type.trim();
    if enc.eq_ignore_ascii_case("STARTTLS") {
        return true;
    }
    if enc.eq_ignore_ascii_case("TLS") {
        return true;
    }
    false
}
