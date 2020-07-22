pub mod binary_writer{
use std::io::BufWriter;
    use std::io::Error;
    use std::fs::File;
    use std::path::Path;
    use std::mem::transmute;
    use std::io::Write;
    use std::hash::Hasher;
    use crate::pmx_types::pmx_types::{PMXVertex, PMXVertexWeight, PMXFace};
    use crate::pmx_types::pmx_types::{Vec2,Vec3,Vec4};

    pub struct BinaryWriter {
       pub inner:BufWriter<File>
    }
    macro_rules! write_bin{
    ($F:ident,$T:ty)=>{
        pub fn $F(&mut self,value:$T){
        let mut buf=[0u8;std::mem::size_of::<$T>()];
        unsafe{buf=transmute(value)};
            self.inner.write(&buf).unwrap();
        }
    };
}
impl BinaryWriter{
    pub fn create<P:AsRef<Path>>(path:P)->Result<BinaryWriter,Error>{
     //   let file = File::open(&path);
        let file=File::create(&path);
        let file_size = std::fs::metadata(&path).unwrap().len();

        match file {
            Ok(file) => {
                let inner = BufWriter::with_capacity(file_size as usize, file);
                Ok(BinaryWriter { inner })
            }
            Err(err) => Err(err)
        }
    }
    pub fn write_vec(&mut self, v: &[u8]) {
        self.inner.write(&v).unwrap();
    }

    pub fn write_text_buf(&mut self, text:&str){
        let len=text.len();
         self.write_i32(len as i32);
        self.write_vec(text.as_bytes());
    }

    pub fn write_vertex_index(&mut self, size: u8, value:u32) {
        match size {
            1=>{self.write_u8(value as u8)}
            2=>{self.write_u16(value as u16)}
            4=>{self.write_u32(value)}
            _=>{}
        }
    }
    pub fn write_sized(&mut self, size: u8, value:i32) {
        match size {
            1=>{ self.write_i8(value as i8); }
            2=>{ self.write_i16(value as i16); }
            4=>{ self.write_i32(value);}
            _=>{}
        }
    }
    pub(crate) fn write_face(&mut self,s_vertex_index:u8,face:PMXFace){
        self.write_vertex_index(s_vertex_index,face.vertices[0]);
        self.write_vertex_index(s_vertex_index,face.vertices[1]);
        self.write_vertex_index(s_vertex_index,face.vertices[2]);
    }

    pub(crate) fn write_pmx_vertex(&mut self, additional_uvs:u8,vertex:PMXVertex,s_bone_index:u8){
        
        self.write_vec3(vertex.position);
        self.write_vec3(vertex.norm);
        self.write_vec2(vertex.uv);
        if additional_uvs > 0 {
            for i in 0..additional_uvs {
                self.write_vec4(vertex.add_uv[i as usize]);
            }
        }

        let weight_type=match vertex.weight_type{
            PMXVertexWeight::BDEF1 => {0},
            PMXVertexWeight::BDEF2 => {1},
            PMXVertexWeight::BDEF4 => {2},
            PMXVertexWeight::SDEF => {3},
            PMXVertexWeight::QDEF => {4},
        };
        self.write_u8(weight_type);
        match vertex.weight_type{
            PMXVertexWeight::BDEF1 => {
                self.write_sized(s_bone_index,vertex.bone_indices[0]);
            },
            PMXVertexWeight::BDEF2 => {
                self.write_sized(s_bone_index,vertex.bone_indices[0]);
                self.write_sized(s_bone_index,vertex.bone_indices[1]);
                self.write_f32(vertex.bone_weights[0]);
            },
            PMXVertexWeight::BDEF4 => {

                self.write_sized(s_bone_index,vertex.bone_indices[0]);
                self.write_sized(s_bone_index,vertex.bone_indices[1]);
                self.write_sized(s_bone_index,vertex.bone_indices[2]);
                self.write_sized(s_bone_index,vertex.bone_indices[3]);
                self.write_f32(vertex.bone_weights[0]);
                self.write_f32(vertex.bone_weights[1]);
                self.write_f32(vertex.bone_weights[2]);
                self.write_f32(vertex.bone_weights[3]);
            },
            PMXVertexWeight::SDEF => {
                self.write_sized(s_bone_index,vertex.bone_indices[0]);
                self.write_sized(s_bone_index,vertex.bone_indices[1]);
                self.write_f32(vertex.bone_weights[0]);
                self.write_vec3(vertex.sdef_c);
                self.write_vec3(vertex.sdef_r0);
                self.write_vec3(vertex.sdef_r1);
            },
            PMXVertexWeight::QDEF => {
                self.write_sized(s_bone_index,vertex.bone_indices[0]);
                self.write_sized(s_bone_index,vertex.bone_indices[1]);
                self.write_sized(s_bone_index,vertex.bone_indices[2]);
                self.write_sized(s_bone_index,vertex.bone_indices[3]);
                self.write_f32(vertex.bone_weights[0]);
                self.write_f32(vertex.bone_weights[1]);
                self.write_f32(vertex.bone_weights[2]);
                self.write_f32(vertex.bone_weights[3]);
            },
        }

        self.write_f32(vertex.edge_mag);
    }
    write_bin!(write_vec4, Vec4);
    write_bin!(write_vec3, Vec3);
    write_bin!(write_vec2, Vec2);
    write_bin!(write_f32, f32);
    write_bin!(write_i32, i32);
    write_bin!(write_u32, u32);
    write_bin!(write_i16, i16);
    write_bin!(write_u16, u16);
    write_bin!(write_i8, i8);
    write_bin!(write_u8, u8);
}
}
