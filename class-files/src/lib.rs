use std::io::{self, Read, Seek};

pub mod bytes;
pub mod descriptors;
pub mod types;

use anyhow::{bail, ensure, Context};
use bytes::ReadNum;
use types::{
    raw::{RawAttribute, RawConstant, RawField, RawMethod},
    resolved::{Attribute, Field, Method},
    ClassAccessFlags, MethodAccessFlags,
};

#[derive(Debug, Clone, Default)]
pub struct ClassFile {
    /// (major, minor)
    pub version: (u16, u16),
    pub constant_pool: Vec<RawConstant>,
    pub access_flags: ClassAccessFlags,
    this_class: usize,
    super_class: usize,
    interfaces: Vec<usize>,
    fields: Vec<RawField>,
    methods: Vec<RawMethod>,
    attributes: Vec<RawAttribute>,
}

impl ClassFile {
    pub fn this_class(&self) -> anyhow::Result<&'_ str> {
        match &self.constant_pool[self.this_class - 1] {
            RawConstant::Class { name_index } => {
                Ok(&self.constant_pool[name_index - 1].unwrap_utf8())
            }
            c => bail!(
                "Expected Class Constant, got {:?} at {}",
                c,
                self.this_class - 1
            ),
        }
    }

    pub fn super_class(&self) -> anyhow::Result<&'_ str> {
        Ok(match &self.constant_pool[self.super_class - 1] {
            RawConstant::Class { name_index } => &self.constant_pool[name_index - 1],
            c => bail!(
                "Expected Class Constant, got {:?} at {}",
                c,
                self.super_class - 1
            ),
        }
        .unwrap_utf8())
    }

    pub fn interfaces(&self) -> impl Iterator<Item = &RawConstant> {
        self.interfaces.iter().map(|n| &self.constant_pool[*n])
    }

    pub fn methods(&self) -> impl Iterator<Item = Method> {
        self.methods
            .iter()
            .map(|m| Method::from_raw(m, &self.constant_pool))
    }

    pub fn fields(&self) -> impl Iterator<Item = Field> {
        self.fields
            .iter()
            .map(|r| Field::from_raw(&r, &self.constant_pool))
    }

    pub fn attributes(&self) -> impl Iterator<Item = Attribute<'_>> {
        self.attributes
            .iter()
            .map(|r| Attribute::from_raw(r, &self.constant_pool))
    }

    pub fn find_entry_point(&self) -> Option<Method> {
        let method = self
            .methods()
            .find(|m| m.name == "main" && m.descriptor == "([Ljava/lang/String;)V")?;

        if (!method.access_flags).intersects(MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC)
        {
            return None;
        }

        Some(method)
    }

    pub fn find_init_method(&self) -> Option<Method> {
        let method = self
            .methods()
            .find(|m| m.name == "<clinit>" && m.descriptor == "()V")?;

        if (!method.access_flags).intersects(MethodAccessFlags::STATIC) {
            return None;
        }

        Some(method)
    }

    pub fn read_from<R>(r: &mut R) -> anyhow::Result<Self>
    where
        R: Read + Seek,
    {
        let mut out: Self = Default::default();

        let magic = r.read_u32().context("parsing magic")?;
        if magic != 0xcafe_babe {
            bail!("Invalid magic value: 0x{:x}", magic);
        }
        // eprintln!("magic = 0x{:X}", magic);

        let minor_version = r.read_u16().context("parsing minor version")?;
        let major_version = r.read_u16().context("parsing major version")?;
        out.version = (major_version, minor_version);

        let constant_pool_count = r.read_u16()?;
        out.constant_pool
            .reserve_exact(constant_pool_count as usize - 1);

        // idk why this counts from one, but java is gonna java...
        let mut i = 1;
        while i < constant_pool_count {
            let (c, skip_next) = RawConstant::read_from(r).context("parsing raw constant")?;
            out.constant_pool.push(c);
            if skip_next {
                i += 1;
                // push an empty value so indexing still works
                out.constant_pool.push(RawConstant::Unused);
            }
            i += 1;
        }

        let access_flags = r.read_u16().context("parsing access_flags")?;
        out.access_flags = ClassAccessFlags::from_bits_retain(access_flags);
        out.this_class = r.read_u16().context("parsing this_class")?.into();
        out.super_class = r.read_u16().context("parsing super_class")?.into();

        macro_rules! read_vec {
            ($vec: expr, $code: block, $dbg: expr) => {
                let count = r.read_u16().context(concat!("parsing count for ", $dbg))?;
                $vec.reserve_exact(count.into());
                for _ in 0..count {
                    $vec.push($code);
                }
            };
            ($vec: expr, $struct: ident) => {
                read_vec!(
                    $vec,
                    { $struct::read_from(r).context(concat!("parsing ", stringify!($ty)))? },
                    stringify!($ty)
                );
            };
        }

        read_vec!(
            out.interfaces,
            { r.read_u16().context("parsing interface")?.into() },
            "Interface"
        );

        read_vec!(out.fields, RawField);
        read_vec!(out.methods, RawMethod);
        read_vec!(out.attributes, RawAttribute);

        // check that we've consumed all bytes
        let remaining_bytes = r.bytes().count();
        ensure!(
            remaining_bytes == 0,
            "{} bytes remaining in file",
            remaining_bytes
        );

        Ok(out)
    }
}
