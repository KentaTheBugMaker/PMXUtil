use crate::binary_writer::BinaryWriter;
use crate::pmx_types::{
    PMXBone, PMXFace, PMXFrame, PMXJoint, PMXJointType, PMXMaterial, PMXModelInfo, PMXMorph,
    PMXRigid, PMXVertex, PMXVertexWeight,
};
use std::io::Write;
use std::path::Path;
/// This is PMXWriter.
/// This hold all PMX ingredient Vertex Face Texture Path etc.
/// When write was called all data was wrote and drop self.
/// ```rust
/// let mut writer=pmx_writer::PMXWriter::begin_writer("/path/to/pmxfile");
/// writer.set_model_info(Some("Name "),Some("Name international"),Some("Comment"),Some("Comment international"));
///writer.set_additional_uv(4)// vertices contains 4 additional uv
///writer.add_vertices(&[vertices]);
///writer.write()
/// ```
pub struct PMXWriter {
    inner: BinaryWriter,
    model_info: Option<PMXModelInfo>,
    vertices: Vec<PMXVertex>,
    additional_uvs: Option<u8>,
    faces: Vec<PMXFace>,
    textures: Vec<String>,
    materials: Vec<PMXMaterial>,
    bones: Vec<PMXBone>,
    morphs: Vec<PMXMorph>,
    frames: Vec<PMXFrame>,
    rigid_bodies: Vec<PMXRigid>,
    joints: Vec<PMXJoint>,
}
impl PMXWriter {
    /// Set model name and start builder
    /// But actually data does not be wrote
    pub fn begin_writer<P: AsRef<Path>>(path: P, is_utf16: bool) -> Self {
        let inner = BinaryWriter::create(path, is_utf16).unwrap();
        Self {
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
        }
    }
    pub fn set_model_info(
        &mut self,
        model_name: Option<&str>,
        model_name_en: Option<&str>,
        comment: Option<&str>,
        comment_en: Option<&str>,
    ) {
        let name = model_name.unwrap_or("").to_string();
        let name_en = model_name_en.unwrap_or("").to_string();
        let comment = comment.unwrap_or("").to_string();
        let comment_en = comment_en.unwrap_or("").to_string();
        let model_info = PMXModelInfo {
            name,
            name_en,
            comment,
            comment_en,
        };
        self.model_info = Some(model_info);
    }
    pub fn set_additional_uv(&mut self, count: u8) -> Result<(), &str> {
        if count > 4 {
            Err("additional uv count is invalid")
        } else {
            self.additional_uvs = Some(count);
            Ok(())
        }
    }

    pub fn add_vertices(&mut self, vertices: &[PMXVertex]) {
        self.vertices.extend_from_slice(vertices);
    }

    pub fn add_faces(&mut self, faces: &[PMXFace]) {
        self.faces.extend_from_slice(faces);
    }

    pub fn add_textures(&mut self, textures: &[String]) {
        self.textures.extend_from_slice(textures);
    }

    pub fn add_materials(&mut self, materials: &[PMXMaterial]) {
        self.materials.extend_from_slice(materials);
    }

    pub fn add_morphs(&mut self, morphs: &[PMXMorph]) {
        self.morphs.extend_from_slice(morphs)
    }

    pub fn add_bones(&mut self, bones: &[PMXBone]) {
        self.bones.extend_from_slice(bones)
    }

    pub fn add_frames(&mut self, frames: &[PMXFrame]) {
        self.frames.extend_from_slice(frames)
    }

    pub fn add_rigid_bodies(&mut self, rigid_bodies: &[PMXRigid]) {
        self.rigid_bodies.extend_from_slice(rigid_bodies)
    }

    pub fn add_joints(&mut self, joints: &[PMXJoint]) {
        self.joints.extend_from_slice(joints)
    }

    /// Actually write data because index size optimization
    /// and drop all
    pub fn write(data_set: Self) {
        // determine we need to use 2.1 extension

        let vertex = data_set
            .vertices
            .iter()
            .find(|vertex| matches!(vertex.weight_type, PMXVertexWeight::QDEF { .. }));
        let morph = data_set.morphs.iter().find(|morph| morph.morph_type > 8);
        let joint = data_set.joints.iter().find(|joint| {
            matches!(
                joint.joint_type,
                PMXJointType::Slider { .. }
                    | PMXJointType::_6DOF { .. }
                    | PMXJointType::ConeTwist { .. }
                    | PMXJointType::Hinge { .. }
                    | PMXJointType::P2P { .. }
            )
        });
        let ext_2_1 = vertex.is_some() | morph.is_some() | joint.is_some();
        println!("2.1Extension:{}", ext_2_1);
        // generate header
        let magic = b"PMX ";
        let version = if ext_2_1 { 2.1 } else { 2.0 };
        let length = 8u8;
        let s_vertex_index = require_bytes(data_set.vertices.len());
        let s_texture_index = require_bytes_signed(data_set.textures.len());
        let s_material_index = require_bytes_signed(data_set.materials.len());
        let s_bone_index = require_bytes_signed(data_set.bones.len());
        let s_morph_index = require_bytes_signed(data_set.morphs.len());
        let s_rigid_body_index = require_bytes_signed(data_set.rigid_bodies.len());
        let parameters = [
            if data_set.inner.is_utf16 { 0 } else { 1 },
            data_set.additional_uvs.unwrap_or(0),
            s_vertex_index,
            s_texture_index,
            s_material_index,
            s_bone_index,
            s_morph_index,
            s_rigid_body_index,
        ];
        println!("s_rigid_body_index {}", s_rigid_body_index);
        //write header
        let mut writer = data_set.inner;
        writer.write_vec(magic);
        writer.write_f32(version);
        writer.write_u8(length);
        writer.write_vec(&parameters);
        //wrote header

        //write model info
        let model_info = data_set.model_info.unwrap();
        writer.write_text_buf(&model_info.name);
        writer.write_text_buf(&model_info.name_en);
        writer.write_text_buf(&model_info.comment);
        writer.write_text_buf(&model_info.comment_en);
        //wrote model info

        writer.write_i32(data_set.vertices.len() as i32);
        for vertex in data_set.vertices {
            writer.write_pmx_vertex(data_set.additional_uvs.unwrap_or(0), vertex, s_bone_index);
        }

        writer.write_i32(3 * data_set.faces.len() as i32);
        for face in data_set.faces {
            writer.write_face(s_vertex_index, face)
        }

        writer.write_i32(data_set.textures.len() as i32);
        for name in data_set.textures {
            writer.write_text_buf(&name);
        }

        writer.write_i32(data_set.materials.len() as i32);
        for material in data_set.materials {
            writer.write_pmx_material(s_texture_index, material)
        }

        writer.write_i32(data_set.bones.len() as i32);
        for bone in data_set.bones {
            writer.write_bone(s_bone_index, bone);
        }

        writer.write_i32(data_set.morphs.len() as i32);
        for morph in data_set.morphs {
            writer.write_pmx_morph(
                s_vertex_index,
                s_bone_index,
                s_material_index,
                s_morph_index,
                s_rigid_body_index,
                morph,
            );
        }

        writer.write_i32(data_set.frames.len() as i32);
        for frame in data_set.frames {
            writer.write_pmx_frame(s_bone_index, s_morph_index, frame);
        }

        writer.write_i32(data_set.rigid_bodies.len() as i32);
        for rigid in data_set.rigid_bodies {
            writer.write_pmx_rigid(s_bone_index, rigid)
        }

        writer.write_i32(data_set.joints.len() as i32);
        for joint in data_set.joints {
            writer.write_pmx_joint(s_rigid_body_index, joint)
        }

        writer.inner.flush();
    }
}
fn require_bytes(len: usize) -> u8 {
    if len < 0xff {
        1 //8 bit
    } else if len < 0xffff {
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
