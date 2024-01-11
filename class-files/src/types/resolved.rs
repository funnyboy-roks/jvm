use std::io::{self, Cursor, Seek};

use crate::bytes::ReadNum;

use super::{raw::*, NestedClassAccessFlags};
use super::{FieldAccessFlags, MethodAccessFlags};

#[derive(Debug, Clone)]
pub enum Constant<'a> {
    Class {
        name: &'a str,
    },
    FieldRef {
        class: &'a Constant<'a>,
        name_and_type: &'a Constant<'a>,
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

#[derive(Debug, Clone)]
pub struct Method<'a> {
    pub access_flags: MethodAccessFlags,
    pub name: &'a str,
    pub descriptor: &'a str,
    pub attributes: &'a [RawAttribute],
    pub constant_pool: &'a [RawConstant],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Exception {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct InnerClassInfo<'a> {
    inner_class_info: &'a RawConstant,
    outer_class_info: &'a RawConstant,
    inner_name: &'a str,
    inner_class_access_flags: NestedClassAccessFlags,
}

#[derive(Debug, Clone, Copy)]
pub struct LineNumber {
    start_pc: usize,
    line_number: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalVariable<'a> {
    start_pc: usize,
    length: usize,
    name: &'a str,
    descriptor: &'a str,
    index: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalVariableType<'a> {
    start_pc: usize,
    length: usize,
    name: &'a str,
    signature: &'a str,
    index: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct AnnotationElement<'a> {
    name: &'a str,
    // TODO:
    // value: AnnotationElementValue<'a>,
    // See <https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7.16.1>
}

#[derive(Debug, Clone)]
pub struct Annotation<'a> {
    /// Field descriptor representing the annotation type corresponding to the annotation
    /// represented by this annotation structure
    ty: &'a str,
    /// Each value of the `elements` table represents a single element-value pair in this
    /// `annotation`.
    elements: Vec<AnnotationElement<'a>>,
}

#[derive(Debug, Clone)]
pub struct BootstrapMethod<'a> {
    method_ref: &'a RawConstant,
    arguments: Vec<&'a RawConstant>,
}

#[derive(Debug, Clone)]
pub enum Attribute<'a> {
    ConstantValue {
        value: &'a RawConstant,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: &'a [u8],
        exception_table: Vec<Exception>,
        // TODO: recursively parse this so we can use Attribute here
        attributes: Vec<RawAttribute>,
    },
    // TODO: Impl these:
    StackMapTable, // TODO
    Exceptions {
        /// Each value in the `exception_index_table` array must be a valid index into the
        /// `constant_pool` table. The constant_pool entry referenced by each table item must be a
        /// Class_info representing a class type that this method is declared to throw.
        exception_index_table: Vec<u16>,
    },
    InnerClasses {
        classes: Vec<InnerClassInfo<'a>>,
    },
    EnclosingMethod {
        class: &'a RawConstant,
        method_index: &'a RawConstant,
    },
    Synthetic,
    Signature {
        signature: &'a RawConstant,
    },
    SourceFile {
        sourcefile: &'a str,
    },
    SourceDebugExtension {
        debug_extension: &'a [u8],
    },
    LineNumberTable {
        table: Vec<LineNumber>,
    },
    LocalVariableTable {
        table: Vec<LocalVariable<'a>>,
    },
    LocalVariableTypeTable {
        table: Vec<LocalVariableType<'a>>,
    },
    Deprecated,
    RuntimeVisibleAnnotations {
        // TODO: annotations: Vec<Annotation<'a>>,
    },
    RuntimeInvisibleAnnotations {
        // TODO
    },
    RuntimeVisibleParameterAnnotations {
        // TODO
    },
    RuntimeInvisibleParameterAnnotations {
        // TODO
    },
    AnnotationDefault {
        // TODO
    },
    /// The `BootstrapMethods` attribute records bootstrap method specifiers referenced by `invokedynamic` instructions
    BootstrapMethods {
        methods: Vec<BootstrapMethod<'a>>,
    },
    Other {
        name: &'a str,
        info: &'a [u8],
    },
}

impl<'a> Attribute<'a> {
    pub fn from_raw(raw: &'a RawAttribute, const_pool: &'a [RawConstant]) -> Self {
        let name = const_pool[raw.attribute_name_index - 1].unwrap_utf8();
        let mut cursor = Cursor::new(&raw.info);
        // TODO: Remove these unwraps
        match name {
            "ConstantValue" => Self::ConstantValue {
                value: &const_pool[cursor.read_u16().unwrap() as usize],
            },
            "Code" => {
                let max_stack = cursor.read_u16().unwrap();
                let max_locals = cursor.read_u16().unwrap();
                let code_length = cursor.read_u32().unwrap();
                let code = &raw.info[cursor.position() as usize..][..code_length as usize];
                cursor
                    .seek(io::SeekFrom::Current(code_length as i64))
                    .unwrap();

                let exception_table_len = cursor.read_u16().unwrap();
                let mut exception_table = Vec::with_capacity(exception_table_len.into());
                for _ in 0..exception_table_len {
                    exception_table.push(Exception {
                        start_pc: cursor.read_u16().unwrap(),
                        end_pc: cursor.read_u16().unwrap(),
                        handler_pc: cursor.read_u16().unwrap(),
                        catch_type: cursor.read_u16().unwrap(),
                    });
                }

                let attributes_count = cursor.read_u16().unwrap();
                let mut attributes = Vec::with_capacity(attributes_count.into());
                for _ in 0..attributes_count {
                    let raw = RawAttribute::read_from(&mut cursor).unwrap();
                    attributes.push(raw);
                }

                Self::Code {
                    max_stack,
                    max_locals,
                    code,
                    exception_table,
                    attributes,
                }
            }
            // TODO: Impl these:
            "StackMapTable" => Self::StackMapTable {},
            "Exceptions" => Self::Exceptions {
                exception_index_table: (0..cursor.read_u16().unwrap())
                    .map(|_| cursor.read_u16().unwrap())
                    .collect(),
            },
            "InnerClasses" => {
                let classes = (0..cursor.read_u16().unwrap())
                    .map(|_| {
                        let inner_class_info = &const_pool[cursor.read_u16().unwrap() as usize - 1];
                        let outer_class_info = &const_pool[cursor.read_u16().unwrap() as usize - 1];
                        let inner_name =
                            &const_pool[cursor.read_u16().unwrap() as usize - 1].unwrap_utf8();
                        let access_flags = cursor.read_u16().unwrap();
                        InnerClassInfo {
                            inner_class_info,
                            outer_class_info,
                            inner_name,
                            inner_class_access_flags: NestedClassAccessFlags::from_bits(
                                access_flags,
                            )
                            .unwrap_or_else(|| {
                                panic!("Invalid Class Access Flags: 0x{:x}", access_flags)
                            }),
                        }
                    })
                    .collect();
                Self::InnerClasses { classes }
                //Self::InnerClasses { info: &raw.info }
            }
            "EnclosingMethod" => Self::EnclosingMethod {
                class: &const_pool[cursor.read_u16().unwrap() as usize - 1],
                method_index: &const_pool[cursor.read_u16().unwrap() as usize - 1],
            },
            "Synthetic" => Self::Synthetic,
            "Signature" => Self::Signature {
                signature: &const_pool[cursor.read_u16().unwrap() as usize - 1],
            },
            "SourceFile" => Self::SourceFile {
                sourcefile: &const_pool[cursor.read_u16().unwrap() as usize - 1].unwrap_utf8(),
            },
            "SourceDebugExtension" => Self::SourceDebugExtension {
                debug_extension: &raw.info,
            },
            "LineNumberTable" => Self::LineNumberTable {
                table: (0..cursor.read_u16().unwrap())
                    .map(|_| LineNumber {
                        start_pc: cursor.read_u16().unwrap().into(),
                        line_number: cursor.read_u16().unwrap().into(),
                    })
                    .collect(),
            },
            "LocalVariableTable" => Self::LocalVariableTable {
                table: (0..cursor.read_u16().unwrap())
                    .map(|_| LocalVariable {
                        start_pc: cursor.read_u16().unwrap().into(),
                        length: cursor.read_u16().unwrap().into(),
                        name: &const_pool[cursor.read_u16().unwrap() as usize - 1].unwrap_utf8(),
                        descriptor: &const_pool[cursor.read_u16().unwrap() as usize - 1]
                            .unwrap_utf8(),
                        index: cursor.read_u16().unwrap().into(),
                    })
                    .collect(),
            },
            "LocalVariableTypeTable" => Self::LocalVariableTypeTable {
                table: (0..cursor.read_u16().unwrap())
                    .map(|_| LocalVariableType {
                        start_pc: cursor.read_u16().unwrap().into(),
                        length: cursor.read_u16().unwrap().into(),
                        name: &const_pool[cursor.read_u16().unwrap() as usize - 1].unwrap_utf8(),
                        signature: &const_pool[cursor.read_u16().unwrap() as usize - 1]
                            .unwrap_utf8(),
                        index: cursor.read_u16().unwrap().into(),
                    })
                    .collect(),
            },
            "Deprecated" => Self::Deprecated,

            // TODO: Annotations
            "RuntimeVisibleAnnotations" => Self::RuntimeVisibleAnnotations {},
            "RuntimeInvisibleAnnotations" => Self::RuntimeInvisibleAnnotations {},
            "RuntimeVisibleParameterAnnotations" => Self::RuntimeVisibleParameterAnnotations {},
            "RuntimeInvisibleParameterAnnotations" => Self::RuntimeInvisibleParameterAnnotations {},
            "AnnotationDefault" => Self::AnnotationDefault {},

            "BootstrapMethods" => Self::BootstrapMethods {
                methods: (0..cursor.read_u16().unwrap())
                    .map(|_| BootstrapMethod {
                        method_ref: &const_pool[cursor.read_u16().unwrap() as usize - 1],
                        arguments: (0..cursor.read_u16().unwrap())
                            .map(|_| &const_pool[cursor.read_u16().unwrap() as usize - 1])
                            .collect(),
                    })
                    .collect(),
            },
            a => {
                eprintln!("Unknown attribute {}", a);
                Self::Other {
                    name: a,
                    info: &raw.info,
                }
            }
        }
    }
}

impl<'a> Method<'a> {
    pub(crate) fn from_raw(raw: &'a RawMethod, constant_pool: &'a [RawConstant]) -> Self {
        Self {
            access_flags: raw.access_flags,
            name: constant_pool[raw.name_index - 1].unwrap_utf8(),
            descriptor: constant_pool[raw.descriptor_index - 1].unwrap_utf8(),
            attributes: &raw.attributes,
            constant_pool,
        }
    }
    pub fn attributes(&self) -> impl Iterator<Item = Attribute<'_>> {
        self.attributes
            .iter()
            .map(|r| Attribute::from_raw(r, self.constant_pool))
    }

    pub fn code(&self) -> Option<Attribute> {
        self.attributes
            .iter()
            .map(|r| Attribute::from_raw(r, self.constant_pool))
            .find(|a| matches!(a, Attribute::Code { .. }))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Field<'a> {
    pub access_flags: FieldAccessFlags,
    pub name: &'a str,
    pub descriptor: &'a str,
    attributes: &'a [RawAttribute],
    constant_pool: &'a [RawConstant],
}

impl<'a> Field<'a> {
    pub(crate) fn from_raw(raw: &'a RawField, const_pool: &'a [RawConstant]) -> Self {
        Self {
            access_flags: raw.access_flags,
            name: const_pool[raw.name_index - 1].unwrap_utf8(),
            descriptor: const_pool[raw.descriptor_index - 1].unwrap_utf8(),
            attributes: &raw.attributes,
            constant_pool: const_pool,
        }
    }

    pub fn attributes(&self) -> impl Iterator<Item = Attribute<'_>> {
        self.attributes
            .iter()
            .map(|r| Attribute::from_raw(r, self.constant_pool))
    }
}
