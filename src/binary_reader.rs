extern crate encoding;

use std::fs::File;
use std::intrinsics::transmute;
use std::io::{BufReader, Error, Read};
use std::path::Path;

use crate::pmx_types::pmx_types::{Encode, PMXHeaderC, Vec2, Vec3, Vec4};

use self::encoding::{DecoderTrap, Encoding};

pub struct BinaryReader {
    inner: BufReader<File>
}

impl BinaryReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<BinaryReader, Error> {
        let file = File::open(&path);
        let file_size = std::fs::metadata(&path).unwrap().len();

        match file {
            Ok(file) => {
                let inner = BufReader::with_capacity(file_size as usize, file);
                Ok(BinaryReader {
                    inner
                })
            }
            Err(err) => { Err(err) }
        }
    }
    pub fn read_vec(&mut self, n: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(n);
        v.resize(n, 0u8);
        self.inner.read_exact(&mut v).unwrap();
        v
    }
    pub fn read_text_buf(&mut self, encode: Encode) -> String {
        let length = self.read_i32();
        let v = self.read_vec(length as usize);
        match encode {
            Encode::UTF8 => {
                String::from_utf8(v).unwrap()
            }
            Encode::Utf16Le => {
                encoding::all::UTF_16LE.decode(&v, DecoderTrap::Strict).unwrap()
            }
        }
    }

    pub fn read_vertex_index(&mut self, n: u8) -> Option<u32> {
        match n {
            1 => {
                Some(self.read_u8() as u32)
            }
            2 => {
                Some(self.read_u16() as u32)
            }
            4 => {
                Some(self.read_i32() as u32)
            }
            _ => {
                None
            }
        }
    }
    pub fn read_sized(&mut self, n: u8) -> Option<i32> {
        match n {
            1 => {
                let tmp = self.read_u8();
                if tmp != 0xff {
                    Some(tmp as i32)
                } else { Some(-1) }
            }
            2 => {
                let tmp = self.read_u16();
                if tmp != 0xffff {
                    Some(tmp as i32)
                } else { Some(-1) }
            }
            4 => {
                let tmp = self.read_u32();
                Some(tmp as i32)
            }
            _ => {
                None
            }
        }
    }


    read_bin!(read_vec4,Vec4);
    read_bin!(read_vec3,Vec3);
    read_bin!(read_vec2,Vec2);
    read_bin!(read_PMXHeader_raw,PMXHeaderC);
    read_bin!(read_f32,f32);
    read_bin!(read_i32,i32);
    read_bin!(read_u32,u32);
    read_bin!(read_i16,i16);
    read_bin!(read_u16,u16);
    read_bin!(read_i8,i8);
    read_bin!(read_u8,u8);
}
