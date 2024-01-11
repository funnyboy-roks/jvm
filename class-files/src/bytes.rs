use anyhow::Context;
use std::io::Read;

macro_rules! read_num_dec {
    ($name: ident -> $ty: ty) => {
        fn $name(&mut self) -> anyhow::Result<$ty>;
    };
}

pub trait ReadNum: Read {
    read_num_dec!(read_u8 -> u8);
    read_num_dec!(read_i8 -> i8);
    read_num_dec!(read_u16 -> u16);
    read_num_dec!(read_i16 -> i16);
    read_num_dec!(read_u32 -> u32);
    read_num_dec!(read_i32 -> i32);
    read_num_dec!(read_u64 -> u64);
    read_num_dec!(read_i64 -> i64);
    read_num_dec!(read_u128 -> u128);
    read_num_dec!(read_i128 -> i128);

    read_num_dec!(read_f32 -> f32);
    read_num_dec!(read_f64 -> f64);
}

macro_rules! read_num_impl {
    ($name: ident -> $ty: ty) => {
        fn $name(&mut self) -> anyhow::Result<$ty> {
            let mut buf = [0u8; std::mem::size_of::<$ty>()];
            self.read_exact(&mut buf)
                .context(concat!("parsing ", stringify!($ty)))?;
            Ok(<$ty>::from_be_bytes(buf))
        }
    };
}

impl<R> ReadNum for R
where
    R: Read,
{
    read_num_impl!(read_u8 -> u8);
    read_num_impl!(read_i8 -> i8);

    read_num_impl!(read_u16 -> u16);
    read_num_impl!(read_i16 -> i16);

    read_num_impl!(read_u32 -> u32);
    read_num_impl!(read_i32 -> i32);

    read_num_impl!(read_u64 -> u64);
    read_num_impl!(read_i64 -> i64);

    read_num_impl!(read_u128 -> u128);
    read_num_impl!(read_i128 -> i128);

    read_num_impl!(read_f32 -> f32);
    read_num_impl!(read_f64 -> f64);
}
