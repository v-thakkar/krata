use crate::bus::{XsdBusError, XsdSocket};
use crate::sys::{
    XSD_DIRECTORY, XSD_MKDIR, XSD_READ, XSD_RM, XSD_TRANSACTION_END, XSD_TRANSACTION_START,
    XSD_WRITE,
};
use std::ffi::CString;

pub struct XsdClient {
    pub socket: XsdSocket,
}

pub trait XsdInterface {
    fn list(&mut self, path: &str) -> Result<Vec<String>, XsdBusError>;
    fn read(&mut self, path: &str) -> Result<Vec<u8>, XsdBusError>;
    fn write(&mut self, path: &str, data: Vec<u8>) -> Result<bool, XsdBusError>;
    fn mkdir(&mut self, path: &str) -> Result<bool, XsdBusError>;
    fn rm(&mut self, path: &str) -> Result<bool, XsdBusError>;
}

impl XsdClient {
    pub fn new() -> Result<XsdClient, XsdBusError> {
        let socket = XsdSocket::dial()?;
        Ok(XsdClient { socket })
    }

    fn list(&mut self, tx: u32, path: &str) -> Result<Vec<String>, XsdBusError> {
        let response = self.socket.send_single(tx, XSD_DIRECTORY, path)?;
        response.parse_string_vec()
    }

    fn read(&mut self, tx: u32, path: &str) -> Result<Vec<u8>, XsdBusError> {
        let response = self.socket.send_single(tx, XSD_READ, path)?;
        Ok(response.payload)
    }

    fn write(&mut self, tx: u32, path: &str, data: Vec<u8>) -> Result<bool, XsdBusError> {
        let mut buffer = Vec::new();
        let path = CString::new(path)?;
        buffer.extend_from_slice(path.as_bytes_with_nul());
        buffer.extend_from_slice(data.as_slice());
        let response = self.socket.send(tx, XSD_WRITE, buffer.as_slice())?;
        response.parse_bool()
    }

    fn mkdir(&mut self, tx: u32, path: &str) -> Result<bool, XsdBusError> {
        self.socket.send_single(tx, XSD_MKDIR, path)?.parse_bool()
    }

    fn rm(&mut self, tx: u32, path: &str) -> Result<bool, XsdBusError> {
        self.socket.send_single(tx, XSD_RM, path)?.parse_bool()
    }

    pub fn transaction(&mut self) -> Result<XsdTransaction, XsdBusError> {
        let response = self.socket.send(0, XSD_TRANSACTION_START, &[])?;
        let tx = response.parse_string()?.parse::<u32>()?;
        Ok(XsdTransaction { client: self, tx })
    }
}

pub struct XsdTransaction<'a> {
    client: &'a mut XsdClient,
    tx: u32,
}

impl XsdInterface for XsdClient {
    fn list(&mut self, path: &str) -> Result<Vec<String>, XsdBusError> {
        self.list(0, path)
    }

    fn read(&mut self, path: &str) -> Result<Vec<u8>, XsdBusError> {
        self.read(0, path)
    }

    fn write(&mut self, path: &str, data: Vec<u8>) -> Result<bool, XsdBusError> {
        self.write(0, path, data)
    }

    fn mkdir(&mut self, path: &str) -> Result<bool, XsdBusError> {
        self.mkdir(0, path)
    }

    fn rm(&mut self, path: &str) -> Result<bool, XsdBusError> {
        self.rm(0, path)
    }
}

impl XsdInterface for XsdTransaction<'_> {
    fn list(&mut self, path: &str) -> Result<Vec<String>, XsdBusError> {
        self.client.list(self.tx, path)
    }

    fn read(&mut self, path: &str) -> Result<Vec<u8>, XsdBusError> {
        self.client.read(self.tx, path)
    }

    fn write(&mut self, path: &str, data: Vec<u8>) -> Result<bool, XsdBusError> {
        self.client.write(self.tx, path, data)
    }

    fn mkdir(&mut self, path: &str) -> Result<bool, XsdBusError> {
        self.client.mkdir(self.tx, path)
    }

    fn rm(&mut self, path: &str) -> Result<bool, XsdBusError> {
        self.client.rm(self.tx, path)
    }
}

impl XsdTransaction<'_> {
    pub fn end(&mut self, abort: bool) -> Result<bool, XsdBusError> {
        let abort_str = if abort { "F" } else { "T" };

        self.client
            .socket
            .send_single(self.tx, XSD_TRANSACTION_END, abort_str)?
            .parse_bool()
    }

    pub fn commit(&mut self) -> Result<bool, XsdBusError> {
        self.end(false)
    }

    pub fn abort(&mut self) -> Result<bool, XsdBusError> {
        self.end(true)
    }
}
