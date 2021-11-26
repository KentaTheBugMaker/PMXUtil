use crate::types::{Encode, HeaderRaw, IndexKinds, Vec2, Vec3, Vec4, VertexIndexKinds};
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::mem::transmute;
use std::path::Path;

macro_rules! read_bin {
    ($F:ident,$T:ty) => {
        pub(crate) fn $F(&mut self) -> $T {
            let mut buf = [0_u8; std::mem::size_of::<$T>()];
            self.inner.read_exact(&mut buf).unwrap();
            unsafe { transmute(buf) }
        }
    };
}

pub(crate) struct BinaryReader {
    inner: BufReader<File>,
}

impl BinaryReader {
    pub(crate) fn open<P: AsRef<Path>>(path: P) -> Result<BinaryReader, Error> {
        let file = File::open(&path);
        let file_size = std::fs::metadata(&path).unwrap().len();

        match file {
            Ok(file) => {
                let inner = BufReader::with_capacity(file_size as usize, file);
                Ok(BinaryReader { inner })
            }
            Err(err) => Err(err),
        }
    }
    pub(crate) fn read_vec(&mut self, n: usize) -> Vec<u8> {
        let mut v = vec![0; n];
        self.inner.read_exact(&mut v).unwrap();
        v
    }
    pub(crate) fn read_text_buf(&mut self, encode: Encode) -> String {
        let length = self.read_i32();
        let v = self.read_vec(length as usize);
        match encode {
            Encode::UTF8 => String::from_utf8(v).unwrap(),
            Encode::Utf16Le => encoding_rs::UTF_16LE.decode(&v).0.to_string(),
        }
    }

    pub(crate) fn read_vertex_index(&mut self, types: VertexIndexKinds) -> i32 {
        match types {
            VertexIndexKinds::U8 => self.read_u8() as i32,
            VertexIndexKinds::U16 => self.read_u16() as i32,
            VertexIndexKinds::I32 => self.read_i32(),
        }
    }

    pub(crate) fn read_sized(&mut self, types: IndexKinds) -> i32 {
        match types {
            IndexKinds::I8 => self.read_i8() as i32,
            IndexKinds::I16 => self.read_i16() as i32,
            IndexKinds::I32 => self.read_i32(),
        }
    }
    read_bin!(read_vec4, Vec4);
    read_bin!(read_vec3, Vec3);
    read_bin!(read_vec2, Vec2);
    read_bin!(read_raw_header, HeaderRaw);
    read_bin!(read_f32, f32);
    read_bin!(read_i32, i32);
    read_bin!(read_i16, i16);
    read_bin!(read_u16, u16);
    read_bin!(read_i8, i8);
    read_bin!(read_u8, u8);
}
