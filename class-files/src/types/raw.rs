use anyhow::bail;

use super::{super::bytes::ReadNum, FieldAccessFlags, MethodAccessFlags};
use std::io::{self, Read};

#[derive(Debug, Clone)]
pub enum RawConstant {
    /// Doubles and Longs are two entries into the constant table, so we place this into the vec to
    /// allow lookups to stay O(1)
    Unused,
    Class {
        name_index: usize,
    },
    FieldRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    MethodRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    InterfaceMethodRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    String {
        string_index: usize,
    },
    Integer {
        num: i32,
    },
    Float {
        num: f32,
    },
    Long {
        num: i64,
    },
    Double {
        num: f64,
    },
    NameAndType {
        name_index: usize,
        descriptor_index: usize,
    },
    Utf8 {
        string: String,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: usize,
    },
    MethodType {
        descriptor_index: usize,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: usize,
        name_and_type_index: usize,
    },
}

impl RawConstant {
    pub fn read_from<R>(r: &mut R) -> anyhow::Result<(Self, bool)>
    where
        R: Read,
    {
        let mut skip_next = false;
        let constant = match r.read_u8()? {
            7 => Self::Class {
                name_index: r.read_u16()?.into(),
            },

            9 => Self::FieldRef {
                class_index: r.read_u16()?.into(),
                name_and_type_index: r.read_u16()?.into(),
            },

            10 => Self::MethodRef {
                class_index: r.read_u16()?.into(),
                name_and_type_index: r.read_u16()?.into(),
            },
            11 => Self::InterfaceMethodRef {
                class_index: r.read_u16()?.into(),
                name_and_type_index: r.read_u16()?.into(),
            },
            8 => Self::String {
                string_index: r.read_u16()?.into(),
            },
            3 => Self::Integer { num: r.read_i32()? },
            4 => Self::Float { num: r.read_f32()? },
            5 => {
                skip_next = true;
                Self::Long { num: r.read_i64()? }
            }
            6 => {
                skip_next = true;
                Self::Double { num: r.read_f64()? }
            }
            12 => Self::NameAndType {
                name_index: r.read_u16()?.into(),
                descriptor_index: r.read_u16()?.into(),
            },
            1 => {
                let count = r.read_u16()?;
                let mut bytes = vec![0u8; count.into()];
                r.read_exact(&mut bytes)?;
                // string: String::from_utf8(bytes.clone())
                //     .with_context(|| format!("bytes: {:x?}", &bytes))?,
                Self::Utf8 {
                    string: cesu8::from_java_cesu8(&bytes)?.to_string(),
                }
            }
            15 => Self::MethodHandle {
                reference_kind: r.read_u8()?,
                reference_index: r.read_u16()?.into(),
            },
            16 => Self::MethodType {
                descriptor_index: r.read_u16()?.into(),
            },
            18 => Self::InvokeDynamic {
                bootstrap_method_attr_index: r.read_u16()?.into(),
                name_and_type_index: r.read_u16()?.into(),
            },
            tag => {
                bail!("Invalid constant tag: {}", tag);
            }
        };

        Ok((constant, skip_next))
    }

    pub fn unwrap_utf8(&self) -> &str {
        match self {
            RawConstant::Utf8 { string } => string,
            _ => unreachable!("unwrap_utf8 on non-utf8 value. was: {:?}", self),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RawField {
    pub(crate) access_flags: FieldAccessFlags,
    pub(crate) name_index: usize,
    pub(crate) descriptor_index: usize,
    pub(crate) attributes: Vec<RawAttribute>,
}

impl RawField {
    pub fn read_from<R>(r: &mut R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let mut out: Self = Default::default();

        let access_flags = r.read_u16()?;
        out.access_flags = FieldAccessFlags::from_bits(access_flags).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid Field Access Flags: 0x{:x}", access_flags),
            )
        })?;

        out.name_index = r.read_u16()?.into();
        out.descriptor_index = r.read_u16()?.into();
        let attribute_count = r.read_u16()?;
        out.attributes.reserve_exact(attribute_count.into());
        for _ in 0..attribute_count {
            out.attributes.push(RawAttribute::read_from(r)?);
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RawAttribute {
    pub(crate) attribute_name_index: usize,
    pub(crate) info: Vec<u8>,
}

impl RawAttribute {
    pub fn read_from<R>(r: &mut R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let mut out: Self = Default::default();
        out.attribute_name_index = r.read_u16()?.into();
        let len = r.read_u32()?;
        out.info = (0..len).map(|_| 0).collect();
        r.read_exact(&mut out.info)?;
        Ok(out)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RawMethod {
    pub(crate) access_flags: MethodAccessFlags,
    pub(crate) name_index: usize,
    pub(crate) descriptor_index: usize,
    pub(crate) attributes: Vec<RawAttribute>,
}

impl RawMethod {
    pub fn read_from<R>(r: &mut R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let mut out: Self = Default::default();

        let access_flags = r.read_u16()?;
        out.access_flags = MethodAccessFlags::from_bits(access_flags).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid Method Access Flags: 0x{:x}", access_flags),
            )
        })?;

        out.name_index = r.read_u16()?.into();
        out.descriptor_index = r.read_u16()?.into();

        let attribute_count = r.read_u16()?;
        out.attributes.reserve_exact(attribute_count.into());
        for _ in 0..attribute_count {
            out.attributes.push(RawAttribute::read_from(r)?);
        }

        Ok(out)
    }
}
