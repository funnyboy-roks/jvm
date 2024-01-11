use std::str::{Chars, FromStr};

use anyhow::{bail, ensure, Context};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldType {
    /// B
    Byte,
    /// C
    Char,
    /// D
    Double,
    /// F
    Float,
    /// I
    Int,
    /// J
    Long,
    /// L<ClassName>;
    ObjReference(String),
    /// S
    Short,
    /// Z
    Boolean,
    /// [
    ArrReference(Box<FieldType>),
}

impl FieldType {
    pub fn from_chars(id: char, chars: &mut Chars) -> anyhow::Result<Self> {
        Ok(match id {
            'B' => Self::Byte,
            'C' => Self::Char,
            'D' => Self::Double,
            'F' => Self::Float,
            'I' => Self::Int,
            'J' => Self::Long,
            'L' => {
                let mut s = String::new();
                let mut c = chars.next().context("Invalid format")?;
                while c != ';' {
                    s.push(c);
                    c = chars.next().context("Invalid format")?;
                }
                Self::ObjReference(s)
            }
            'S' => Self::Short,
            'Z' => Self::Boolean,
            '[' => Self::ArrReference(Box::new(Self::from_chars(
                chars.next().context("Invalid format")?,
                chars,
            )?)),
            c => bail!("Invalid type, found: '{}'", c),
        })
    }
}

impl ToString for FieldType {
    fn to_string(&self) -> String {
        let mut s = String::new();
        match self {
            FieldType::Byte => s.push('B'),
            FieldType::Char => s.push('C'),
            FieldType::Double => s.push('D'),
            FieldType::Float => s.push('F'),
            FieldType::Int => s.push('I'),
            FieldType::Long => s.push('J'),
            FieldType::ObjReference(or) => {
                s.push('L');
                s.push_str(&or);
                s.push(';');
            }
            FieldType::Short => s.push('S'),
            FieldType::Boolean => s.push('Z'),
            FieldType::ArrReference(ar) => {
                s.push('[');
                s.push_str(&ar.to_string());
            }
        }
        s
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnDescriptor {
    FieldType(FieldType),
    Void,
}

impl ReturnDescriptor {
    pub fn from_chars(id: char, chars: &mut Chars) -> anyhow::Result<Self> {
        Ok(match id {
            'V' => Self::Void,
            _ => Self::FieldType(FieldType::from_chars(id, chars)?),
        })
    }
}
impl ToString for ReturnDescriptor {
    fn to_string(&self) -> String {
        match self {
            ReturnDescriptor::FieldType(ft) => return ft.to_string(),
            ReturnDescriptor::Void => "V".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub params: Vec<FieldType>,
    pub return_value: ReturnDescriptor,
}

impl FromStr for MethodDescriptor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        ensure!(chars.next() == Some('('), "Invalid format -- expected '('");

        let mut id = chars
            .next()
            .context("Invalid format -- expected identifier")?;
        let mut params = Vec::new();
        while id != ')' {
            params.push(FieldType::from_chars(id, &mut chars)?);
            id = chars
                .next()
                .context("Invalid format -- expected more chars")?;
        }
        dbg!(&params);
        id = chars
            .next()
            .context("Invalid format -- expected more chars")?;

        Ok(Self {
            params,
            return_value: ReturnDescriptor::from_chars(id, &mut chars)?,
        })
    }
}

impl ToString for MethodDescriptor {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push('(');
        for param in &self.params {
            s.push_str(&param.to_string());
        }
        s.push(')');
        s.push_str(&self.return_value.to_string());
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn docs_example() {
        let descriptor = "(IDLjava/lang/Thread;)Ljava/lang/Object;";

        let md = MethodDescriptor::from_str(descriptor).unwrap();
        let expected = MethodDescriptor {
            params: vec![
                FieldType::Int,
                FieldType::Double,
                FieldType::ObjReference("java/lang/Thread".into()),
            ],
            return_value: ReturnDescriptor::FieldType(FieldType::ObjReference(
                "java/lang/Object".into(),
            )),
        };

        assert_eq!(md, expected);
        assert_eq!(md.to_string(), descriptor);
    }

    #[test]
    fn test2() {
        let descriptor = "([[B)V";

        let md = MethodDescriptor::from_str(descriptor).unwrap();
        let expected = MethodDescriptor {
            params: vec![FieldType::ArrReference(Box::new(FieldType::ArrReference(
                Box::new(FieldType::Byte),
            )))],
            return_value: ReturnDescriptor::Void,
        };

        assert_eq!(md, expected);
        assert_eq!(md.to_string(), descriptor);
    }
}
