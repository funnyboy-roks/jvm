use class_files::{
    descriptors::MethodDescriptor,
    types::resolved::{Attribute, Method},
};

pub mod java {
    pub type Boolean = bool;
    pub type Byte = i8;
    pub type Char = u16;
    pub type Short = i16;
    pub type Int = i32;
    pub type Float = f32;
    pub type Long = i64;
    pub type Double = f64;
}

/// [^note]: See <https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-2.html#jvms-2.3>
#[derive(Copy, Clone, Debug)]
pub enum DataType {
    /// Default: false
    Boolean(bool),
    /// Default: 0
    Byte(i8),
    /// Default: '\u000'
    Char(u16),
    /// Default: 0
    Short(i16),
    /// Default: 0
    Int(i32),
    /// Default: +0.0
    Float(f32),
    /// Default: 0
    Long(i64),
    /// Default: +0.0
    Double(f64),
    /// Reference to a class instance
    /// Value references an index into the heap
    ClassReference(usize),
    /// Reference to an array
    /// Value references an index into the Heap
    ArrayReference(usize),
    /// No default
    /// Value references an index into the Heap
    InterfaceReference(usize),
    /// The values of the `ReturnAddr` type are pointers to the opcodes of Java Virtual Machine
    /// instructions. Unlike the numeric primitive types, the `ReturnArrd` type does not
    /// correspond to any Java programming language type and cannot be modified by the running
    /// program.
    /// `pc` value of the return
    ReturnAddr(usize),
    /// Special `null` value -- not considered a primitive
    Null,

    /// Special internal value for the default value in an array -- if access, the default value
    /// for the expected type should be resolved
    Empty,
}

macro_rules! from_dt {
    ($ty: ty => $dt: ident) => {
        impl From<$ty> for DataType {
            fn from(value: $ty) -> Self {
                DataType::$dt(value)
            }
        }
    };
}

from_dt!(bool => Boolean);
from_dt!(i8 => Byte);
from_dt!(u16 => Char);
from_dt!(i16 => Short);
from_dt!(i32 => Int);
from_dt!(f32 => Float);
from_dt!(i64 => Long);
from_dt!(f64 => Double);

impl DataType {
    pub fn is_primitive(&self) -> bool {
        match self {
            DataType::Byte(_)
            | DataType::Short(_)
            | DataType::Int(_)
            | DataType::Long(_)
            | DataType::Char(_)
            | DataType::Float(_)
            | DataType::Double(_)
            | DataType::Boolean(_) => true,

            DataType::Null
            | DataType::ClassReference(_)
            | DataType::ArrayReference { .. }
            | DataType::InterfaceReference(_)
            | DataType::ReturnAddr(_)
            | DataType::Empty => false,
        }
    }

    /// [^ref]: See https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.11.1-320
    pub fn get_computation_type(&self) -> Self {
        match self {
            DataType::Boolean(b) => DataType::Int((*b).into()),
            DataType::Byte(b) => DataType::Int((*b).into()),
            DataType::Char(c) => DataType::Int((*c).into()),
            DataType::Short(s) => DataType::Int((*s).into()),
            DataType::Int(_) => self.clone(),
            DataType::Float(_) => self.clone(),
            DataType::Long(_) => self.clone(),
            DataType::Double(_) => self.clone(),
            DataType::ClassReference(_) => self.clone(),
            DataType::ArrayReference { .. } => self.clone(),
            DataType::InterfaceReference(_) => self.clone(),
            DataType::ReturnAddr(_) => self.clone(),
            DataType::Null => self.clone(),
            DataType::Empty => self.clone(),
        }
    }
}

/// [^ref]: See <https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-2.html#jvms-2.6>
#[derive(Clone, Debug)]
pub(crate) struct StackFrame {
    pub(crate) variables: Vec<DataType>,
    pub(crate) op_stack: Vec<DataType>,
    pub(crate) pc: usize,
}

impl StackFrame {
    pub(crate) fn new(max_stack: u16, max_locals: u16) -> Self {
        Self {
            variables: vec![DataType::Empty; max_locals.into()],
            op_stack: Vec::with_capacity(max_stack.into()),
            pc: 0,
        }
    }

    pub(crate) fn for_method(method: &Method) -> Self {
        let Some(Attribute::Code {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        }) = method.code()
        else {
            unreachable!()
        };

        let md: MethodDescriptor = method.descriptor.parse().unwrap();
        dbg!(md);

        let variables = vec![DataType::Empty; max_locals.into()];

        Self {
            variables,
            op_stack: Vec::with_capacity(max_stack.into()),
            pc: 0,
        }
    }
}
