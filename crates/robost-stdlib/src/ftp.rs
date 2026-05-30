use crate::{get_str, NodeError, NodeResult};
use serde_json::Value;
use std::collections::HashMap;

/// Internal enum to unify plain FTP and FTPS (TLS) streams behind the same
/// method names without requiring boxing/dyn dispatch.
enum Conn {
    Plain(suppaftp::FtpStream),
    Tls(suppaftp::NativeTlsFtpStream),
}

macro_rules! ftp_call {
    ($conn:expr, $method:ident $(, $arg:expr)*) => {
        match &mut $conn {
            Conn::Plain(s) => s.$method($($arg),*),
            Conn::Tls(s)   => s.$method($($arg),*),
        }
    };
}

struct FtpParams {
    host: String,
    port: u16,
    user: String,
    password: String,
    tls: bool,
}

impl FtpParams {
    fn from_inputs(inputs: &HashMap<String, Value>) -> Result<Self, NodeError> {
        Ok(Self {
            host: get_str(inputs, "host")?,
            port: inputs.get("port").and_then(|v| v.as_u64()).unwrap_or(21) as u16,
            user: get_str(inputs, "user")?,
            password: get_str(inputs, "password")?,
            tls: inputs.get("tls").and_then(|v| v.as_bool()).unwrap_or(true),
        })
    }

    fn connect(&self) -> Result<Conn, NodeError> {
        connect(&self.host, self.port, &self.user, &self.password, self.tls)
    }
}

fn connect(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    tls: bool,
) -> Result<Conn, NodeError> {
    let addr = format!("{host}:{port}");
    if tls {
        let ftp = suppaftp::NativeTlsFtpStream::connect(&addr)
            .map_err(|e| NodeError::Other(format!("ftp connect failed: {e}")))?;
        let connector = suppaftp::native_tls::TlsConnector::new()
            .map_err(|e| NodeError::Other(format!("ftp tls connector: {e}")))?;
        let mut ftp = ftp
            .into_secure(suppaftp::NativeTlsConnector::from(connector), host)
            .map_err(|e| NodeError::Other(format!("ftp tls failed: {e}")))?;
        ftp.login(user, password)
            .map_err(|e| NodeError::Other(format!("ftp login failed: {e}")))?;
        Ok(Conn::Tls(ftp))
    } else {
        let mut ftp = suppaftp::FtpStream::connect(&addr)
            .map_err(|e| NodeError::Other(format!("ftp connect failed: {e}")))?;
        ftp.login(user, password)
            .map_err(|e| NodeError::Other(format!("ftp login failed: {e}")))?;
        Ok(Conn::Plain(ftp))
    }
}

pub fn upload(inputs: HashMap<String, Value>) -> NodeResult {
    let p = FtpParams::from_inputs(&inputs)?;
    let local = get_str(&inputs, "local")?;
    let remote = get_str(&inputs, "remote")?;

    let mut ftp = p.connect()?;
    let data = std::fs::read(&local)
        .map_err(|e| NodeError::Other(format!("ftp upload read local: {e}")))?;
    let mut cursor = std::io::Cursor::new(data);
    ftp_call!(ftp, put_file, &remote, &mut cursor)
        .map_err(|e| NodeError::Other(format!("ftp upload failed: {e}")))?;
    ftp_call!(ftp, quit).ok();
    tracing::info!(local, remote, "ftp.upload");
    Ok(HashMap::new())
}

pub fn download(inputs: HashMap<String, Value>) -> NodeResult {
    let p = FtpParams::from_inputs(&inputs)?;
    let remote = get_str(&inputs, "remote")?;
    let local = get_str(&inputs, "local")?;

    let mut ftp = p.connect()?;
    let data = ftp_call!(ftp, retr_as_buffer, &remote)
        .map_err(|e| NodeError::Other(format!("ftp download failed: {e}")))?;
    if let Some(parent) = std::path::Path::new(&local).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| NodeError::Other(format!("ftp download mkdir: {e}")))?;
        }
    }
    std::fs::write(&local, data.into_inner())
        .map_err(|e| NodeError::Other(format!("ftp download write: {e}")))?;
    ftp_call!(ftp, quit).ok();
    tracing::info!(remote, local, "ftp.download");
    Ok(HashMap::new())
}

pub fn list(inputs: HashMap<String, Value>) -> NodeResult {
    let p = FtpParams::from_inputs(&inputs)?;
    let remote = get_str(&inputs, "remote")?;

    let mut ftp = p.connect()?;
    let entries = ftp_call!(ftp, list, Some(remote.as_str()))
        .map_err(|e| NodeError::Other(format!("ftp list failed: {e}")))?;
    ftp_call!(ftp, quit).ok();
    let files: Vec<Value> = entries.into_iter().map(Value::String).collect();
    tracing::info!(count = files.len(), "ftp.list");
    let mut out = HashMap::new();
    out.insert("files".to_owned(), Value::Array(files));
    Ok(out)
}

pub fn delete(inputs: HashMap<String, Value>) -> NodeResult {
    let p = FtpParams::from_inputs(&inputs)?;
    let remote = get_str(&inputs, "remote")?;

    let mut ftp = p.connect()?;
    ftp_call!(ftp, rm, &remote).map_err(|e| NodeError::Other(format!("ftp delete failed: {e}")))?;
    ftp_call!(ftp, quit).ok();
    tracing::info!(remote, "ftp.delete");
    Ok(HashMap::new())
}

pub fn mkdir(inputs: HashMap<String, Value>) -> NodeResult {
    let p = FtpParams::from_inputs(&inputs)?;
    let remote = get_str(&inputs, "remote")?;

    let mut ftp = p.connect()?;
    ftp_call!(ftp, mkdir, &remote)
        .map_err(|e| NodeError::Other(format!("ftp mkdir failed: {e}")))?;
    ftp_call!(ftp, quit).ok();
    tracing::info!(remote, "ftp.mkdir");
    Ok(HashMap::new())
}
