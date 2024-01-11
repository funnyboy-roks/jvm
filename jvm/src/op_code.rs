use std::io::{Read, Seek, SeekFrom};

use anyhow::{bail, ensure, Context};
use class_files::{
    bytes::ReadNum,
    descriptors::MethodDescriptor,
    types::{
        raw::RawConstant,
        resolved::{Attribute, Method},
        MethodAccessFlags,
    },
};

use crate::{
    types::{DataType, StackFrame},
    HeapItem, Jvm,
};

pub(crate) fn handle_op_code<'a, R>(
    instruction: u8,
    jvm: &'a mut Jvm,
    curr_class: &str,
    code: &mut R,
    stack_frame: usize,
) -> anyhow::Result<()>
where
    R: Read + Seek,
{
    let stack_frame = &mut jvm.stack[stack_frame];
    eprintln!("Instruction: 0x{:x}", instruction);
    match instruction {
        0x0 => return Ok(()),
        0x32 => {
            // aaload -- Load `reference` from array -- Like `my_arr[5]`
            eprintln!("\tInstruction: aaload");
            let Some(DataType::Int(idx)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            let Some(arrayref) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let arrayref = match arrayref {
                DataType::ArrayReference(i) => &jvm.heap[i],
                DataType::Null => {
                    todo!("NPE");
                }
                _ => bail!("Invalid stack args"),
            };

            let arrayref = match arrayref {
                HeapItem::Array(arrayref) => arrayref,
                v => {
                    bail!("Expected array, got {:?}", v);
                }
            };

            stack_frame.op_stack.push(arrayref.get(idx as usize));
            return Ok(());
        }
        0x53 => {
            // aastore -- Like `my_arr[5] = 10`
            eprintln!("\tInstruction: aastore");
            let Some(value) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(idx)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            let Some(ref mut arrayref) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            match arrayref {
                DataType::ArrayReference(i) => {
                    let HeapItem::Array(ref mut arrayref) = jvm.heap[*i] else {
                        bail!("not an array");
                    };
                    arrayref.set(idx as usize, value)?;
                }
                DataType::Null => {
                    todo!("NPE");
                }
                _ => bail!("Invalid stack args"),
            }
            return Ok(());
        }
        0x01 => { // aconst_null
        }
        0x19 => {
            // aload
            let n = code.read_u8()?;
            stack_frame.op_stack.push(stack_frame.variables[n as usize]);
            return Ok(());
        }
        0x2a..=0x2d => {
            // aload_<n>
            let n = instruction - 0x2a;
            stack_frame.op_stack.push(stack_frame.variables[n as usize]);
            return Ok(());
        }
        0xbd => { // anewarray
        }
        0xb0 => { // areturn
        }
        0xbe => { // arraylength
        }
        0x3a => {
            // astore
            let Some(value) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            let n = code.read_u8()?;
            stack_frame.variables[n as usize] = value;
            return Ok(());
        }
        0x4b..=0x4e => {
            // astore_<n>
            let Some(value) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            let n = instruction - 0x4b;
            stack_frame.variables[n as usize] = value;
            return Ok(());
        }
        0xbf => { // athrow
        }
        0x33 => {
            // baload
            let Some(DataType::Int(index)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::ArrayReference(arrayref)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame
                .op_stack
                .push(jvm.heap.get_array(arrayref)?.get(index as usize));
            return Ok(());
        }
        0x54 => {
            // bastore
            let Some(DataType::Int(value)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(index)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::ArrayReference(arrayref)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            jvm.heap
                .get_array_mut(arrayref)?
                .set(index as usize, DataType::Int(value))?;
            return Ok(());
        }
        0x10 => {
            // bipush
            let byte = code.read_u8()?;
            stack_frame.op_stack.push(DataType::Int(byte.into()));
            return Ok(());
        }
        0xca => { // breakpoint
        }
        0x34 => { // caload
        }
        0x55 => { // castore
        }
        0xc0 => { // checkcast
        }
        0x90 => { // d2f
        }
        0x8e => { // d2i
        }
        0x8f => { // d2l
        }
        0x63 => { // dadd
        }
        0x31 => { // daload
        }
        0x52 => { // dastore
        }
        0x98 => { // dcmpg
        }
        0x97 => { // dcmpl
        }
        0x0e => { // dconst_0
        }
        0x0f => { // dconst_1
        }
        0x6f => { // ddiv
        }
        0x18 => { // dload
        }
        0x26 => { // dload_0
        }
        0x27 => { // dload_1
        }
        0x28 => { // dload_2
        }
        0x29 => { // dload_3
        }
        0x6b => { // dmul
        }
        0x77 => { // dneg
        }
        0x73 => { // drem
        }
        0xaf => { // dreturn
        }
        0x39 => { // dstore
        }
        0x47 => { // dstore_0
        }
        0x48 => { // dstore_1
        }
        0x49 => { // dstore_2
        }
        0x4a => { // dstore_3
        }
        0x67 => { // dsub
        }
        0x59 => {
            // dup
            let top = stack_frame.op_stack.pop().context("No value on stack")?;
            stack_frame.op_stack.push(top);
            stack_frame.op_stack.push(top);
            return Ok(());
        }
        0x5a => { // dup_x1
        }
        0x5b => { // dup_x2
        }
        0x5c => { // dup2
        }
        0x5d => { // dup2_x1
        }
        0x5e => { // dup2_x2
        }
        0x8d => { // f2d
        }
        0x8b => { // f2i
        }
        0x8c => { // f2l
        }
        0x62 => { // fadd
        }
        0x30 => {
            // faload
            let Some(DataType::Int(index)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::ArrayReference(arrayref)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame
                .op_stack
                .push(jvm.heap.get_array(arrayref)?.get(index as usize));
            return Ok(());
        }
        0x51 => {
            // fastore
            let Some(DataType::Float(value)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(index)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::ArrayReference(arrayref)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            jvm.heap
                .get_array_mut(arrayref)?
                .set(index as usize, DataType::Float(value))?;
            return Ok(());
        }
        0x96 => { // fcmpg
        }
        0x95 => { // fcmpl
        }
        0x0b..=0x0d => {
            // fconst_0
            let val = (instruction - 0xb) as f32;
            stack_frame.op_stack.push(DataType::Float(val));
            return Ok(());
        }
        0x6e => { // fdiv
        }
        0x17 => { // fload
        }
        0x22 => { // fload_0
        }
        0x23 => { // fload_1
        }
        0x24 => { // fload_2
        }
        0x25 => { // fload_3
        }
        0x6a => { // fmul
        }
        0x76 => { // fneg
        }
        0x72 => { // frem
        }
        0xae => { // freturn
        }
        0x38 => {
            // fstore
            let idx = code.read_u8()?;
            eprintln!("\tInstruction: fstore {}", idx);
            let Some(value) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            stack_frame.variables[idx as usize] = value;
            return Ok(());
        }
        0x43..=0x46 => {
            // fstore_<n>
            let n = instruction - 0x43;
            eprintln!("\tInstruction: fstore_{}", n);
            let Some(value) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            stack_frame.variables[n as usize] = value;
            return Ok(());
        }
        0x66 => { // fsub
        }
        0xb4 => { // getfield
        }
        0xb2 => {
            // getstatic -- Get `static` field from class
            let index = code.read_u16()?;
            eprintln!("Unimpled Instruction: getstatic {:02x}", index);

            let class = &jvm.classes[curr_class];
            let (class_index, name_and_type_index) = match &class.constant_pool[index as usize - 1]
            {
                RawConstant::FieldRef {
                    class_index,
                    name_and_type_index,
                } => (class_index, name_and_type_index),
                c => unreachable!("Expected FieldRef, got {:?}", c),
            };
            dbg!(class_index, name_and_type_index);

            let RawConstant::Class { name_index } = class.constant_pool[class_index - 1] else {
                unreachable!();
            };
            let class_name = class.constant_pool[name_index - 1]
                .unwrap_utf8()
                .to_string();

            jvm.init_class(&class_name)?;

            let class = jvm.classes.get_mut(&class_name).context("")?;
            dbg!(&class);
            //let (class_index, name_and_type_index) = match class.constant_pool[class_index - 1] {
            //    RawConstant::MethodRef {
            //        class_index,
            //        name_and_type_index,
            //    } => (class_index, name_and_type_index),
            //    _ => unreachable!(),
            //};
        }
        0xa7 => { // goto
        }
        0xc8 => { // goto_w
        }
        0x91 => { // i2b
        }
        0x92 => { // i2c
        }
        0x87 => { // i2d
        }
        0x86 => { // i2f
        }
        0x85 => { // i2l
        }
        0x93 => { // i2s
        }
        0x60 => {
            // iadd
            eprintln!("\tInstruction: iadd");
            let Some(DataType::Int(a)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(b)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame.op_stack.push(DataType::Int(a + b));
            return Ok(());
        }
        0x2e => {
            // iaload
            let Some(DataType::Int(index)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::ArrayReference(arrayref)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame
                .op_stack
                .push(jvm.heap.get_array(arrayref)?.get(index as usize));
            return Ok(());
        }
        0x7e => {
            // iand
            eprintln!("\tInstruction: iand");
            let Some(DataType::Int(a)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(b)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame.op_stack.push(DataType::Int(a & b));
            return Ok(());
        }
        0x4f => {
            // iastore
            let Some(DataType::Int(value)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(index)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::ArrayReference(arrayref)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            jvm.heap
                .get_array_mut(arrayref)?
                .set(index as usize, DataType::Int(value))?;
            return Ok(());
        }
        0x02..=0x08 => {
            // iconst_<i>
            let i = instruction as i32 - 3;
            eprintln!("\tInstruction: i_const{}", i);
            stack_frame.op_stack.push(DataType::Int(i));
            return Ok(());
        }
        0x6c => {
            // idiv
            eprintln!("\tInstruction: idiv");
            let Some(DataType::Int(a)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(b)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame.op_stack.push(DataType::Int(a / b));
            return Ok(());
        }
        0xa5 => { // if_acmpeq
        }
        0xa6 => { // if_acmpne
        }
        0x9f => { // if_icmpeq
        }
        0xa2 => { // if_icmpge
        }
        0xa3 => { // if_icmpgt
        }
        0xa4 => { // if_icmple
        }
        0xa1 => { // if_icmplt
        }
        0xa0 => { // if_icmpne
        }
        0x99 => { // ifeq
        }
        0x9c => { // ifge
        }
        0x9d => { // ifgt
        }
        0x9e => { // ifle
        }
        0x9b => { // iflt
        }
        0x9a => { // ifne
        }
        0xc7 => { // ifnonnull
        }
        0xc6 => { // ifnull
        }
        0x84 => { // iinc
        }
        0x15 => { // iload
        }
        0x1a..=0x1d => {
            // iload_<n>
            let idx = instruction - 0x1a;
            eprintln!("\tInstruction: iload_{}", idx);
            stack_frame
                .op_stack
                .push(stack_frame.variables[idx as usize]);
            return Ok(());
        }
        0xfe => { // impdep1
        }
        0xff => { // impdep2
        }
        0x68 => {
            // imul
            eprintln!("\tInstruction: imul");
            let Some(DataType::Int(a)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(b)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame.op_stack.push(DataType::Int(a * b));
            return Ok(());
        }
        0x74 => {
            // ineg
            eprintln!("\tInstruction: ineg");
            let Some(DataType::Int(a)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame.op_stack.push(DataType::Int(-a));
            return Ok(());
        }
        0xc1 => { // instanceof
        }
        0xba => { // invokedynamic
        }
        0xb9 => { // invokeinterface
        }
        0xb7 => { // invokespecial
        }
        0xb8 => {
            // invokestatic
            eprintln!("\tInstruction: invokestatic");
            let index = code.read_u16()?;
            dbg!(index);

            let class = &jvm.classes[curr_class];
            let (class, name, descriptor) = match &class.constant_pool[index as usize - 1] {
                RawConstant::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                }
                | RawConstant::MethodRef {
                    class_index,
                    name_and_type_index,
                } => {
                    let method_class = &class.constant_pool[class_index - 1];
                    let method_class = match method_class {
                        RawConstant::Class { name_index } => {
                            class.constant_pool[name_index - 1].unwrap_utf8()
                        }
                        _ => unreachable!(),
                    };
                    let method_class = &jvm.classes[method_class];
                    match &class.constant_pool[name_and_type_index - 1] {
                        RawConstant::NameAndType {
                            name_index,
                            descriptor_index,
                        } => (
                            method_class,
                            class.constant_pool[name_index - 1].unwrap_utf8(),
                            class.constant_pool[descriptor_index - 1].unwrap_utf8(),
                        ),
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            };

            let method = class
                .methods()
                .find(|m| m.name == name && m.descriptor == descriptor)
                .context("Expected method")?;

            dbg!(method.name);

            if method.access_flags.intersects(MethodAccessFlags::NATIVE) {
                eprintln!("NATIVE METHOD");
                // TODO: FIND A BETTER WAY THAN THIS:
                let method = Method {
                    access_flags: method.access_flags,
                    name: &method.name.to_string(),
                    descriptor: &method.name.to_string(),
                    attributes: &method.attributes.to_vec(),
                    constant_pool: &vec![], // easier than reallocating this entire vec
                };
                let name = class.this_class()?.to_string();
                jvm.handle_native_method(&name, &method)?;
                return Ok(());
            }

            let Attribute::Code {
                code,
                exception_table,
                attributes,
                ..
            } = method.code().context("Code attribute not present")?
            else {
                bail!("fu");
            };

            eprintln!("\tcode = {:x?}", code);

            let md: MethodDescriptor = method.descriptor.parse()?;

            let mut new_stack_frame = StackFrame::for_method(&method);

            for i in 0..md.params.len() {
                let v = stack_frame.op_stack.pop().context("")?;
                new_stack_frame.variables[md.params.len() - i - 1] = v;
            }
            dbg!(&new_stack_frame);

            jvm.stack.push(new_stack_frame);

            let code = code.to_vec().into_boxed_slice();
            let class = class.this_class()?.to_string();

            jvm.run_code(&class, &code)?;
            return Ok(());
        }
        0xb6 => { // invokevirtual
        }
        0x80 => {
            // ior
            eprintln!("\tInstruction: ior");
            let Some(DataType::Int(a)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(b)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame.op_stack.push(DataType::Int(a | b));
            return Ok(());
        }
        0x70 => { // irem
        }
        0xac => {
            // ireturn
            eprintln!("\tInstruction: ireturn");
            code.seek(SeekFrom::End(0))?;
            let return_val = stack_frame.op_stack.pop().unwrap();
            dbg!(&jvm.heap);
            dbg!(stack_frame);
            jvm.stack.pop();
            jvm.stack.last_mut().unwrap().op_stack.push(return_val);
            return Ok(());
        }
        0x78 => { // ishl
        }
        0x7a => { // ishr
        }
        0x36 => {
            // istore
            let idx = code.read_u8()?;
            eprintln!("\tInstruction: istore {}", idx);
            let Some(value) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            stack_frame.variables[idx as usize] = value;
            return Ok(());
        }
        0x3b..=0x3e => {
            // istore_<n>
            let idx = instruction - 59;
            eprintln!("\tInstruction: istore_{}", idx);
            let Some(value) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };
            stack_frame.variables[idx as usize] = value;
            return Ok(());
        }
        0x64 => {
            // isub
            eprintln!("\tInstruction: isub");
            let Some(DataType::Int(b)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            let Some(DataType::Int(a)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args")
            };

            stack_frame.op_stack.push(DataType::Int(a - b));
            return Ok(());
        }
        0x7c => { // iushr
        }
        0x82 => { // ixor
        }
        0xa8 => {
            // jsr -- deprecated
            panic!("Unsupported opcode: jsr (0xa8)");
        }
        0xc9 => {
            // jsr_w -- deprecated
            panic!("Unsupported opcode: jsr_w (0xc9)");
        }
        0x8a => { // l2d
        }
        0x89 => { // l2f
        }
        0x88 => { // l2i
        }
        0x61 => { // ladd
        }
        0x2f => { // laload
        }
        0x7f => { // land
        }
        0x50 => { // lastore
        }
        0x94 => { // lcmp
        }
        0x09 => { // lconst_0
        }
        0x0a => { // lconst_1
        }
        0x12 => { // ldc
        }
        0x13 => { // ldc_w
        }
        0x14 => { // ldc2_w
        }
        0x6d => { // ldiv
        }
        0x16 => { // lload
        }
        0x1e => { // lload_0
        }
        0x1f => { // lload_1
        }
        0x20 => { // lload_2
        }
        0x21 => { // lload_3
        }
        0x69 => { // lmul
        }
        0x75 => { // lneg
        }
        0xab => { // lookupswitch
        }
        0x81 => { // lor
        }
        0x71 => { // lrem
        }
        0xad => { // lreturn
        }
        0x79 => { // lshl
        }
        0x7b => { // lshr
        }
        0x37 => { // lstore
        }
        0x3f => { // lstore_0
        }
        0x40 => { // lstore_1
        }
        0x41 => { // lstore_2
        }
        0x42 => { // lstore_3
        }
        0x65 => { // lsub
        }
        0x7d => { // lushr
        }
        0x83 => { // lxor
        }
        0xc2 => { // monitorenter
        }
        0xc3 => { // monitorexit
        }
        0xc5 => { // multianewarray
        }
        0xbb => { // new
        }
        0xbc => {
            // newarray
            let atype = code.read_u8()?;
            let Some(DataType::Int(size)) = stack_frame.op_stack.pop() else {
                bail!("Invalid stack args");
            };
            let array = jvm.heap.create_array(atype, size as usize)?;
            stack_frame.op_stack.push(DataType::ArrayReference(array));
            return Ok(());
        }
        0x57 => {
            // pop
            stack_frame.op_stack.pop();
            return Ok(());
        }
        0x58 => { // pop2
        }
        0xb5 => { // putfield
        }
        0xb3 => { // putstatic
        }
        0xa9 => {
            // ret -- effectively deprecated since jsr and jsr_w are deprecated
            panic!("Unsupported opcode: ret (0xa9)");
        }
        0xb1 => {
            // return
            eprintln!("\tInstruction: return");
            code.seek(SeekFrom::End(0))?;
            dbg!(&jvm.heap);
            dbg!(stack_frame);
            jvm.stack.pop();
            return Ok(());
        }
        0x35 => { // saload
        }
        0x56 => { // sastore
        }
        0x11 => { // sipush
        }
        0x5f => { // swap
        }
        0xaa => { // tableswitch
        }
        0xc4 => { // wide
        }
        0xcb..=0xfd => { // (no name)
        }
    }
    todo!()
}
