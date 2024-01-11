use anyhow::{bail, Context};
use class_files::{
    bytes::ReadNum,
    descriptors::MethodDescriptor,
    types::resolved::{Attribute, Method},
    ClassFile,
};
use op_code::handle_op_code;
use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Cursor, Seek},
    ops::{Deref, Index, IndexMut},
    path::{Path, PathBuf},
};
use types::{java, DataType, StackFrame};

pub mod op_code;
pub mod types;

#[derive(Debug, Clone)]
pub(crate) enum Array {
    Boolean(Box<[java::Boolean]>),
    Char(Box<[java::Char]>),
    Float(Box<[java::Float]>),
    Double(Box<[java::Double]>),
    Byte(Box<[java::Byte]>),
    Short(Box<[java::Short]>),
    Int(Box<[java::Int]>),
    Long(Box<[java::Long]>),
}

macro_rules! slice {
    ($default_value: expr; $count: expr) => {
        vec![$default_value; $count].into_boxed_slice()
    };
}

impl Array {
    fn create(atype: u8, size: usize) -> anyhow::Result<Self> {
        Ok(match atype {
            4 => Self::Boolean(slice![Default::default(); size]),
            5 => Self::Char(slice![Default::default(); size]),
            6 => Self::Float(slice![Default::default(); size]),
            7 => Self::Double(slice![Default::default(); size]),
            8 => Self::Byte(slice![Default::default(); size]),
            9 => Self::Short(slice![Default::default(); size]),
            10 => Self::Int(slice![Default::default(); size]),
            11 => Self::Long(slice![Default::default(); size]),
            _ => bail!("Unknown atype: {}", atype),
        })
    }

    fn get(&self, index: usize) -> DataType {
        match self {
            Array::Boolean(a) => a[index].into(),
            Array::Char(a) => a[index].into(),
            Array::Float(a) => a[index].into(),
            Array::Double(a) => a[index].into(),
            Array::Byte(a) => a[index].into(),
            Array::Short(a) => a[index].into(),
            Array::Int(a) => a[index].into(),
            Array::Long(a) => a[index].into(),
        }
    }

    fn set(&mut self, index: usize, value: DataType) -> anyhow::Result<()> {
        macro_rules! f {
            ($a: ident, $dt: ident) => {{
                let DataType::$dt(b) = value else {
                    bail!(concat!("Can't assign {:?} to ", stringify!($dt)), value);
                };
                $a[index] = b;
            }};
        }
        match self {
            Array::Boolean(a) => {
                match value {
                    DataType::Boolean(b) => a[index] = b,
                    DataType::Int(b) => a[index] = b & 1 != 0,
                    _ => {
                        bail!(concat!("Can't assign {:?} to ", stringify!(Boolean)), value);
                    }
                };
            }
            Array::Byte(a) => {
                match value {
                    DataType::Byte(b) => a[index] = b,
                    DataType::Int(b) => a[index] = (b & 0xff) as java::Byte,
                    _ => {
                        bail!(concat!("Can't assign {:?} to ", stringify!(Boolean)), value);
                    }
                };
            }
            Array::Char(a) => f!(a, Char),
            Array::Float(a) => f!(a, Float),
            Array::Double(a) => f!(a, Double),
            Array::Short(a) => f!(a, Short),
            Array::Int(a) => f!(a, Int),
            Array::Long(a) => f!(a, Long),
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HeapItem {
    Object {
        // TODO
    },
    Primitive(
        // TODO
    ),
    Array(Array),
    Null,
    Empty,
}

impl HeapItem {
    pub fn is_empty(&self) -> bool {
        matches!(self, HeapItem::Empty)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Heap {
    inner: Vec<HeapItem>,
    max_size: usize,
}

impl Default for Heap {
    fn default() -> Self {
        Heap {
            max_size: usize::MAX,
            inner: Default::default(),
        }
    }
}

impl Heap {
    pub fn collect_garbage(&mut self) -> anyhow::Result<()> {
        // TODO: May require JVM to be passed
        //       Look at all stack frames for references into here (op stack & variables)
        //       Referenes are indexes to a non-empty value within bounds
        todo!()
    }

    pub fn is_valid_reference(&self, index: usize) -> bool {
        index < self.inner.len() && !matches!(self.inner.get(index).unwrap(), HeapItem::Empty)
    }

    pub fn remove(&mut self, index: usize) -> anyhow::Result<()> {
        let len = self.inner.len();
        if !self.is_valid_reference(index) {
            bail!("Tried to remove item at invalid index: {}", index);
        }

        let item = self.inner.get_mut(index).unwrap();
        if index == len - 1 {
            self.inner.pop();
        } else {
            _ = std::mem::replace(item, HeapItem::Empty);
        }

        Ok(())
    }

    pub fn create_array(&mut self, atype: u8, size: usize) -> anyhow::Result<usize> {
        let array = HeapItem::Array(Array::create(atype, size)?);
        self.try_append(array)
    }

    fn try_append(&mut self, item: HeapItem) -> anyhow::Result<usize> {
        for (i, it) in self.inner.iter_mut().enumerate() {
            if it.is_empty() {
                _ = std::mem::replace(it, item);
                return Ok(i);
            }
        }
        if self.inner.len() < self.max_size {
            self.inner.push(item);
            Ok(self.inner.len() - 1)
        } else {
            bail!("Max heap size exceeded");
        }
    }

    fn get_array(&self, index: usize) -> anyhow::Result<&Array> {
        let Some(item) = self.inner.get(index) else {
            bail!(
                "Index {} out of bounds for length {}",
                index,
                self.inner.len()
            );
        };

        let HeapItem::Array(arr) = item else {
            bail!("Heap item is not an array: {:?}", item);
        };

        Ok(arr)
    }

    fn get_array_mut(&mut self, index: usize) -> anyhow::Result<&mut Array> {
        let len = self.inner.len();
        let Some(item) = self.inner.get_mut(index) else {
            bail!("Index {} out of bounds for length {}", index, len);
        };

        let HeapItem::Array(ref mut arr) = item else {
            bail!("Heap item is not an array: {:?}", item);
        };

        Ok(arr)
    }
}

impl Index<usize> for Heap {
    type Output = HeapItem;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.index(index)
    }
}

impl IndexMut<usize> for Heap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.inner.index_mut(index)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Class {
    pub(crate) file: ClassFile,
    pub(crate) initialised: bool,
}

impl Class {
    pub fn new(file: ClassFile) -> Self {
        Class {
            file,
            initialised: false,
        }
    }
}

impl Deref for Class {
    type Target = ClassFile;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

pub(crate) struct Jvm<'a> {
    // TODO: this should be different per thread
    // XXX: moved to the individual stack frames, I'm not sure what the intended way to manage
    // functions and stuff is.
    // pub(crate) pc: usize,
    // TODO: this should be different per thread
    // TODO: max stack size (configurable)
    pub(crate) stack: Vec<StackFrame>,
    /// [^see]: <https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.5.3>
    pub(crate) heap: Heap,
    pub(crate) classes: HashMap<String, Class>,
    pub(crate) entry_class: Option<&'a str>,
}

impl<'a> Jvm<'a> {
    pub fn new() -> Self {
        Self {
            stack: Default::default(),
            heap: Default::default(),
            classes: Default::default(),
            entry_class: None,
        }
    }

    pub fn load_class_from_file<P>(&mut self, path: P) -> anyhow::Result<String>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path)?;
        let mut file = BufReader::new(file);
        let class = ClassFile::read_from(&mut file)?;
        let name = class.this_class()?.to_string();
        self.classes.insert(name.clone(), Class::new(class));

        Ok(name)
    }

    pub fn load_classes_from_files<P>(&mut self, paths: &[P]) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        for path in paths {
            let file = fs::File::open(&path)
                .with_context(|| format!("opening {}", path.as_ref().display()))?;
            let mut file = BufReader::new(file);
            let class = ClassFile::read_from(&mut file)
                .with_context(|| format!("parsing {}", path.as_ref().display()))?;
            self.classes
                .insert(class.this_class()?.to_string(), Class::new(class));
        }
        Ok(())
    }

    fn read_dir_recursive<P>(path: P) -> Vec<PathBuf>
    where
        P: AsRef<Path>,
    {
        let Ok(entries) = fs::read_dir(path) else {
            return vec![];
        };
        entries
            .flatten()
            .flat_map(|entry| {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        return Self::read_dir_recursive(&entry.path());
                    } else if ft.is_file() {
                        return vec![entry.path()];
                    }
                }
                vec![]
            })
            .collect()
    }

    pub fn load_classes_from_dir<P>(&mut self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let files = Self::read_dir_recursive(path);
        let files: Vec<_> = files
            .iter()
            .filter(|f| {
                if let Some(file_name) = f.file_name() {
                    let file_name = file_name.to_str().unwrap();
                    file_name != "module-info.class" && file_name.ends_with(".class")
                } else {
                    false
                }
            })
            .collect();
        self.load_classes_from_files(&files)
    }

    pub fn set_entry_class(&mut self, class: &'a str) {
        self.entry_class = Some(class);
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        // check for entry class
        let Some(entry_class) = self.entry_class else {
            bail!("Entry class not set");
        };
        let Some(entry_class) = self.classes.get(entry_class) else {
            bail!("Entry class '{}' not found", entry_class);
        };

        let entry_class = entry_class.clone();

        // find entry point
        let Some(entry_point) = entry_class.find_entry_point() else {
            bail!(
                "No entry point found in class '{}'",
                entry_class.this_class()?
            );
        };

        //dbg!(&entry_point);

        self.run_method(&entry_class, &entry_point)?;
        let stack_frame = self.stack.pop();
        dbg!(stack_frame);

        Ok(())
    }

    fn run_method(&mut self, class: &Class, method: &Method<'_>) -> anyhow::Result<()> {
        let Some(Attribute::Code {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        }) = method.code()
        else {
            bail!("No code attribute for method '{}'", method.name);
        };

        let _: MethodDescriptor = dbg!(method.descriptor.parse()?);

        self.stack.push(StackFrame::new(max_stack, max_locals));

        dbg!(attributes
            .iter()
            .map(|a| Attribute::from_raw(&a, &class.constant_pool))
            .collect::<Vec<_>>());

        dbg!(max_stack, max_locals, code, exception_table, attributes);

        self.run_code(class.this_class()?, code)?;

        Ok(())
    }

    fn run_code(&mut self, curr_class: &str, code: &[u8]) -> anyhow::Result<()> {
        let stack_frame = self.stack.len() - 1;

        let mut cursor = Cursor::new(code);
        loop {
            eprintln!("=> stack_frame: {:?}", &stack_frame);
            let start = self.stack[stack_frame].pc as u64; //self.pc as u64;
            cursor.set_position(start);
            let instruction = cursor.read_u8()?;

            // do things
            handle_op_code(instruction, self, curr_class, &mut cursor, stack_frame)?;

            let dpc = (cursor.seek(std::io::SeekFrom::Current(0))? - start) as usize;
            dbg!(dpc);
            //self.pc += dpc;
            if stack_frame < self.stack.len() {
                self.stack[stack_frame].pc += dpc;

                if self.stack[stack_frame].pc >= code.len() {
                    eprintln!("Out of code (no more code)");
                    break;
                }
            } else {
                eprintln!("Out of code (no more stack)");
                break;
            }
        }
        Ok(())
    }

    /// Initialise the class if it has not been initialised already
    /// Returns whether it was initialised by the calling of this function.
    pub fn init_class(&mut self, class_name: &str) -> anyhow::Result<bool> {
        let class = self.classes.get_mut(class_name).context("")?;

        if class.initialised {
            return Ok(false);
        }

        let class = class.clone();
        let method = class.find_init_method().context("")?;

        self.run_method(&class, &method)?;

        self.classes.get_mut(class_name).unwrap().initialised = true;

        Ok(true)
    }

    pub fn handle_native_method(&mut self, class: &str, method: &Method) -> anyhow::Result<()> {
        eprintln!(
            "Handle native method: class={} method={}",
            class, method.name
        );
        todo!()
    }
}

fn main() -> anyhow::Result<()> {
    let mut jvm = Jvm::new();

    jvm.load_classes_from_dir("stdlib/java.base/java/lang")
        .context("loading std lib")?;

    // TODO: Proper CLI
    jvm.load_classes_from_files(&std::env::args().skip(2).collect::<Vec<_>>())?;

    // TODO: Proper CLI
    let entry_class = jvm.load_class_from_file(std::env::args().nth(1).unwrap())?;

    jvm.set_entry_class(&entry_class);

    jvm.run()?;

    Ok(())
}
