use std::{collections::HashMap, io::Read};

use rmp::{
    Marker,
    decode::{MarkerReadError, RmpRead},
};
use serde::de::DeserializeOwned;
use tracing::instrument;

use crate::{
    errors::{Error, ValueError},
    protocol::Message,
    server::{Object, Response, Value},
};

macro_rules! decode {
    ($reader:expr, $code:expr; $($ty:ident),+) => {
        match $code {
            $(crate::server::$ty::CODE => rmp_serde::from_read::<_, crate::server::$ty>($reader)?.into(),)+
            code => return Err(Error::InvalidCode(code)),
        }
    };
}

pub struct Decoder<R: Read + RmpRead> {
    reader: R,
}

impl<R: Read + RmpRead> Decoder<R>
where
    R: RmpRead<Error = std::io::Error>,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    fn marker(&mut self) -> Result<Marker, MarkerReadError<std::io::Error>> {
        rmp::decode::read_marker(&mut self.reader)
    }

    fn decode_string(&mut self, len: usize) -> Result<String, ValueError> {
        let mut buff = vec![0u8; len as usize];
        self.reader.read_exact(&mut buff)?;

        Ok(String::from_utf8(buff)?)
    }

    #[instrument(skip(self))]
    fn decode_property(&mut self) -> Result<(String, Value), ValueError> {
        let marker = self.marker()?;

        if !matches!(marker, Marker::FixArray(3)) {
            return Err(ValueError::InvalidMarker(marker));
        }

        let code = self.reader.read_data_u8()?;

        match code {
            0x10 => {
                let name: String = self.decode()?.try_into()?;
                let value = self.decode()?;

                Ok((name, value))
            }
            _ => unimplemented!(),
        }
    }

    #[instrument(skip(self))]
    fn decode_array(&mut self, n: usize) -> Result<Value, ValueError> {
        let mut array = Vec::with_capacity(n);

        for _ in 0..n {
            array.push(self.decode()?);
        }

        Ok(Value::Array(array))
    }

    #[instrument(skip(self))]
    fn decode_properties(&mut self, n: usize) -> Result<HashMap<String, Value>, ValueError> {
        let mut properties = HashMap::default();

        for _ in 0..n {
            let (key, value) = self.decode_property()?;
            properties.insert(key, value);
        }

        Ok(properties)
    }

    #[instrument(skip(self))]
    fn decode_inner(&mut self, custom_type: bool) -> Result<Value, ValueError> {
        let marker = self.marker()?;

        match marker {
            Marker::FixArray(_) if custom_type => match self.reader.read_data_u8()? {
                // Typed, Dynamic
                0x1 => {
                    let class_name: String = self.decode_inner(false)?.try_into()?;
                    let module_uri: String = self.decode_inner(false)?.try_into()?;
                    let properties = match self.marker()? {
                        Marker::FixArray(n) => self.decode_properties(n as usize),
                        Marker::Array16 => {
                            let n = self.reader.read_data_u16()?;
                            self.decode_properties(n as usize)
                        }
                        Marker::Array32 => {
                            let n = self.reader.read_data_u32()?;
                            self.decode_properties(n as usize)
                        }
                        marker => Err(ValueError::InvalidMarker(marker)),
                    }?;

                    Ok(Value::Object(Object {
                        class_name,
                        module_uri,
                        properties,
                    }))
                }
                // Mapping
                0x3 => self.decode_inner(false),
                // Listing
                0x5 => self.decode_inner(false),
                // Function
                0xE => Ok(Value::Function),
                c => unimplemented!("code {c} is not implemented"),
            },

            Marker::I8 => Ok(Value::Int(rmp::decode::read_i8(&mut self.reader)? as i64)),
            Marker::I16 => Ok(Value::Int(rmp::decode::read_i16(&mut self.reader)? as i64)),
            Marker::I32 => Ok(Value::Int(rmp::decode::read_i32(&mut self.reader)? as i64)),
            Marker::I64 => Ok(Value::Int(rmp::decode::read_i64(&mut self.reader)?)),
            Marker::U8 => Ok(Value::Uint(rmp::decode::read_u8(&mut self.reader)? as u64)),
            Marker::U16 => Ok(Value::Uint(rmp::decode::read_u16(&mut self.reader)? as u64)),
            Marker::U32 => Ok(Value::Uint(rmp::decode::read_u32(&mut self.reader)? as u64)),
            Marker::U64 => Ok(Value::Uint(rmp::decode::read_u64(&mut self.reader)?)),
            Marker::F32 => Ok(Value::Float(rmp::decode::read_f32(&mut self.reader)? as f64)),
            Marker::F64 => Ok(Value::Float(rmp::decode::read_f64(&mut self.reader)?)),
            Marker::Null => Ok(Value::Null),
            Marker::True => Ok(Value::Bool(true)),
            Marker::False => Ok(Value::Bool(false)),
            Marker::FixStr(size) => Ok(Value::String(self.decode_string(size as usize)?)),
            Marker::FixPos(pos) => Ok(Value::Uint(pos as u64)),
            Marker::Str8 => {
                let len = self.reader.read_data_u8()?;
                Ok(Value::String(self.decode_string(len as usize)?))
            }
            Marker::Str16 => {
                let len = self.reader.read_data_u16()?;
                Ok(Value::String(self.decode_string(len as usize)?))
            }
            Marker::Str32 => {
                let len = self.reader.read_data_u32()?;
                Ok(Value::String(self.decode_string(len as usize)?))
            }
            Marker::FixMap(n) => {
                let mut map = Vec::with_capacity(n as usize);

                for _ in 0..n {
                    let value = self.decode()?;
                    let key = self.decode()?;

                    map.push((key, value));
                }

                Ok(Value::Map(map))
            }
            Marker::Array16 => {
                let n = self.reader.read_data_u16()?;
                self.decode_array(n as usize)
            }
            Marker::Array32 => {
                let n = self.reader.read_data_u32()?;
                self.decode_array(n as usize)
            }
            Marker::FixArray(n) => self.decode_array(n as usize),
            marker => unimplemented!("unknown marker: {marker:#?}"),
        }
    }

    #[instrument(skip(self), err(Debug))]
    pub fn decode(&mut self) -> Result<Value, ValueError> {
        self.decode_inner(true)
    }

    #[instrument(skip(self))]
    pub fn decode_response(&mut self) -> Result<Response, Error> {
        let marker = self.marker()?;

        if !matches!(marker, Marker::FixArray(len) if len == 2) {
            return Err(Error::InvalidMarker(marker));
        }

        let code: u64 = rmp_serde::from_read(&mut self.reader)?;

        Ok(decode!(
            &mut self.reader, code;
            CreateEvaluatorResponse,
            EvaluateResponse,
            Log,
            ReadResourceRequest,
            ReadModuleRequest,
            ListResourcesRequest,
            ListModulesRequest,
            InitializeModuleReaderRequest,
            InitializeResourceReaderRequest,
            CloseExternalProcess
        ))
    }

    pub fn decode_response_typed<T>(&mut self) -> Result<T, Error>
    where
        T: Message + DeserializeOwned,
        T: TryFrom<Response, Error = Error>,
    {
        self.decode_response()?.try_into()
    }
}
