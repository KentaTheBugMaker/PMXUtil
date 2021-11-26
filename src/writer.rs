//! PMX writing module.

use crate::binary_writer::BinaryWriter;
use crate::types::{
    Bone, Face, Frame, Joint, JointType, Material, ModelInfo, Morph, Rigid, SoftBody, Vertex,
    VertexWeight,
};
use std::io::{Error, Write};
use std::path::Path;

/// PMX writer
///
/// This hold all  ingredients e.g. Vertex, Face, Texture Path,
///
/// When write was called all data was wrote and dropped.
/// ```rust
/// use pmx_util::types::ModelInfo;
/// let vertices = vec![];
/// let mut writer=pmx_util::writer::Writer::begin_writer("./path_to_pmx_file.pmx",true).unwrap();
/// writer.set_model_info(&ModelInfo{
/// name:"PMXモデル名".to_owned(),
/// name_en:"A PMX Model Name".to_owned(),
/// comment:"何かコメントをここに".to_owned(),
/// comment_en:"Exported by pmx_util".to_owned(),
/// });
/// writer.set_additional_uv(4);// vertices contains 4 additional uv
/// writer.add_vertices(&vertices);
/// writer.write();
/// ```
pub struct Writer {
    inner: BinaryWriter,
    model_info: Option<ModelInfo>,
    vertices: Vec<Vertex>,
    additional_uvs: Option<u8>,
    faces: Vec<Face>,
    textures: Vec<String>,
    materials: Vec<Material>,
    bones: Vec<Bone>,
    morphs: Vec<Morph>,
    frames: Vec<Frame>,
    rigid_bodies: Vec<Rigid>,
    joints: Vec<Joint>,
    soft_bodies: Vec<SoftBody>,
}
impl Writer {
    ///
    ///
    /// # Arguments
    ///
    /// * `path`: where to write
    /// * `encode_to_utf16`: if true text will encoded in UTF-16 Little Endian
    ///     if you don't have any special reason turn on it to keep MMD compatibility.
    ///
    /// returns: Result<Writer, Error>
    ///
    /// # Errors
    ///  if failed to create file with given path.
    /// # Examples
    ///
    /// ```
    /// use pmx_util::types::ModelInfo;
    /// let vertices = vec![];
    /// let mut writer=pmx_util::writer::Writer::begin_writer("./path_to_pmx_file.pmx",true).unwrap();
    /// ```
    pub fn begin_writer<P: AsRef<Path>>(path: P, encode_to_utf_16: bool) -> Result<Writer, Error> {
        let inner = BinaryWriter::create(path, encode_to_utf_16)?;
        Ok(Self {
            inner,
            model_info: None,
            vertices: vec![],
            additional_uvs: None,
            faces: vec![],
            textures: vec![],
            materials: vec![],
            morphs: vec![],
            bones: vec![],
            rigid_bodies: vec![],
            frames: vec![],
            joints: vec![],
            soft_bodies: vec![],
        })
    }

    pub fn set_model_info(&mut self, model_info: &ModelInfo) {
        self.model_info.replace(model_info.clone());
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `count`: 0..4
    ///
    /// # Errors
    /// if additional uv count exceed 4.
    ///
    pub fn set_additional_uv(&mut self, count: u8) -> Result<(), &str> {
        if count > 4 {
            Err("additional uv count is invalid")
        } else {
            self.additional_uvs = Some(count);
            Ok(())
        }
    }

    pub fn add_vertices(&mut self, vertices: &[Vertex]) {
        self.vertices.extend_from_slice(vertices);
    }

    pub fn add_faces(&mut self, faces: &[Face]) {
        self.faces.extend_from_slice(faces);
    }

    pub fn add_textures(&mut self, textures: &[String]) {
        self.textures.extend_from_slice(textures);
    }

    pub fn add_materials(&mut self, materials: &[Material]) {
        self.materials.extend_from_slice(materials);
    }

    pub fn add_morphs(&mut self, morphs: &[Morph]) {
        self.morphs.extend_from_slice(morphs);
    }

    pub fn add_bones(&mut self, bones: &[Bone]) {
        self.bones.extend_from_slice(bones);
    }

    pub fn add_frames(&mut self, frames: &[Frame]) {
        self.frames.extend_from_slice(frames);
    }

    pub fn add_rigid_bodies(&mut self, rigid_bodies: &[Rigid]) {
        self.rigid_bodies.extend_from_slice(rigid_bodies);
    }

    pub fn add_joints(&mut self, joints: &[Joint]) {
        self.joints.extend_from_slice(joints);
    }

    pub fn add_soft_bodies(&mut self, soft_bodies: &[SoftBody]) {
        self.soft_bodies.extend_from_slice(soft_bodies);
    }

    pub fn write(self) -> Result<(), WritePMXErrors> {
        // determine we need to use 2.1 extension

        let vertex = self
            .vertices
            .iter()
            .find(|vertex| matches!(vertex.weight_type, VertexWeight::QDEF { .. }));
        let morph = self.morphs.iter().find(|morph| morph.morph_type > 8);
        let joint = self.joints.iter().find(|joint| {
            matches!(
                joint.joint_type,
                JointType::Slider { .. }
                    | JointType::SixDof { .. }
                    | JointType::ConeTwist { .. }
                    | JointType::Hinge { .. }
                    | JointType::P2P { .. }
            )
        });
        let ext_2_1 =
            vertex.is_some() | morph.is_some() | joint.is_some() | !self.soft_bodies.is_empty();

        // generate header
        let magic = [0x50_u8, 0x4d, 0x58, 0x20];
        let version = if ext_2_1 { 2.1 } else { 2.0 };
        let length = 8_u8;
        let s_vertex_index = require_bytes_vertex(self.vertices.len());
        let s_texture_index = require_bytes_signed(self.textures.len());
        let s_material_index = require_bytes_signed(self.materials.len());
        let s_bone_index = require_bytes_signed(self.bones.len());
        let s_morph_index = require_bytes_signed(self.morphs.len());
        let s_rigid_body_index = require_bytes_signed(self.rigid_bodies.len());
        let parameters = [
            if self.inner.is_utf16 { 0 } else { 1 },
            self.additional_uvs.unwrap_or(0),
            s_vertex_index,
            s_texture_index,
            s_material_index,
            s_bone_index,
            s_morph_index,
            s_rigid_body_index,
        ];
        //write header
        let mut writer = self.inner;
        writer.write_vec(&magic);
        writer.write_f32(version);
        writer.write_u8(length);
        writer.write_vec(&parameters);
        //wrote header

        //write model info
        let model_info = if let Some(mi) = self.model_info {
            mi
        } else {
            return Err(WritePMXErrors::NoModelInfo);
        };
        writer.write_text_buf(&model_info.name);
        writer.write_text_buf(&model_info.name_en);
        writer.write_text_buf(&model_info.comment);
        writer.write_text_buf(&model_info.comment_en);
        //wrote model info

        writer.write_i32(self.vertices.len() as i32);
        for vertex in self.vertices {
            writer.write_pmx_vertex(self.additional_uvs.unwrap_or(0), &vertex, s_bone_index);
        }

        writer.write_i32(3 * self.faces.len() as i32);
        for face in self.faces {
            writer.write_face(s_vertex_index, face)
        }

        writer.write_i32(self.textures.len() as i32);
        for name in self.textures {
            writer.write_text_buf(&name);
        }

        writer.write_i32(self.materials.len() as i32);
        for material in self.materials {
            writer.write_pmx_material(s_texture_index, &material)
        }

        writer.write_i32(self.bones.len() as i32);
        for bone in self.bones {
            writer.write_bone(s_bone_index, bone);
        }

        writer.write_i32(self.morphs.len() as i32);
        for morph in self.morphs {
            writer.write_pmx_morph(
                s_vertex_index,
                s_bone_index,
                s_material_index,
                s_morph_index,
                s_rigid_body_index,
                morph,
            );
        }

        writer.write_i32(self.frames.len() as i32);
        for frame in self.frames {
            writer.write_pmx_frame(s_bone_index, s_morph_index, frame);
        }

        writer.write_i32(self.rigid_bodies.len() as i32);
        for rigid in self.rigid_bodies {
            writer.write_pmx_rigid(s_bone_index, &rigid)
        }

        writer.write_i32(self.joints.len() as i32);
        for joint in self.joints {
            writer.write_pmx_joint(s_rigid_body_index, &joint)
        }
        // 2.1 extended section.
        if ext_2_1 {
            writer.write_i32(self.soft_bodies.len() as i32);
            for soft_body in self.soft_bodies {
                writer.write_pmx_soft_body(
                    s_material_index,
                    s_rigid_body_index,
                    s_vertex_index,
                    soft_body,
                );
            }
        }
        writer.inner.flush().map_err(WritePMXErrors::IoError)
    }
}

fn require_bytes_vertex(len: usize) -> u8 {
    if len < 0x100 {
        1 //8 bit
    } else if len < 0x10000 {
        2 //16 bit
    } else {
        4 //32 bit
    }
}

fn require_bytes_signed(len: usize) -> u8 {
    if len < 128 {
        1 //8 bit
    } else if len < 32768 {
        2 //16 bit
    } else {
        4 //32 bit
    }
}

pub enum WritePMXErrors {
    NoModelInfo,
    IoError(std::io::Error),
}
