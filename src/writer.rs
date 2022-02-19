//! PMX writing module.
use std::convert::TryFrom;

use crate::binary_writer::BinaryWriter;
use crate::types::{
    Bone, Encode, Face, Frame, Header, IndexKinds, Joint, JointType, Material, ModelInfo, Morph,
    MorphKinds, PMXVersion, Rigid, SoftBody, Vertex, VertexIndexKinds, VertexWeight,
};
use std::io::{Error, Write};
use std::num::TryFromIntError;
use std::path::Path;

/// PMX writer
///
/// This hold all  ingredients e.g. Vertex, Face, Texture Path,
///
/// When write was called all data was wrote and dropped.
///
/// ```rust
/// use PMXUtil::types::ModelInfo;
/// let vertices = vec![];
/// let mut writer=PMXUtil::writer::Writer::begin_writer(true);
/// writer.set_model_info(&ModelInfo{
///     name:"PMXモデル名".to_owned(),
///     name_en:"A PMX Model Name".to_owned(),
///     comment:"何かコメントをここに".to_owned(),
///     comment_en:"Exported by pmx_util".to_owned(),
/// });
/// writer.set_additional_uv(4);// vertices contains 4 additional uv
/// writer.add_vertices(&vertices);
/// writer.write_to_path("./path/to/pmx/file.pmx");
/// ```
pub struct Writer {
    encode_to_utf_16: bool,
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
    /// # Arguments
    ///
    /// * `encode_to_utf16`: if true text will encoded in UTF-16 Little Endian
    ///     if you don't have any special reason turn on it to keep MMD compatibility.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// let mut writer=PMXUtil::writer::Writer::begin_writer(true);
    /// ```
    pub fn begin_writer(encode_to_utf_16: bool) -> Self {
        Self {
            encode_to_utf_16,
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
        }
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
    /// the default additional uv count is 0.
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

    fn calculate_header(&self) -> (Header, bool) {
        let vertex = self
            .vertices
            .iter()
            .find(|vertex| matches!(vertex.weight_type, VertexWeight::QDEF { .. }));
        let morph = self.morphs.iter().find(|morph| {
            matches!(
                morph.morph_data,
                MorphKinds::Flip(_) | MorphKinds::Impulse(_)
            )
        });
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

        // calculate all parameters and create actual writer.
        (
            Header {
                magic: "PMX ".to_owned(),
                version: if ext_2_1 {
                    PMXVersion::V21
                } else {
                    PMXVersion::V20
                },
                length: 8,
                encode: if self.encode_to_utf_16 {
                    Encode::Utf16Le
                } else {
                    Encode::UTF8
                },
                additional_uv: self.additional_uvs.unwrap_or(0),
                s_vertex_index: optimal_data_type_vertex(self.vertices.len()),
                s_texture_index: optimal_data_type(self.textures.len()),
                s_material_index: optimal_data_type(self.materials.len()),
                s_bone_index: optimal_data_type(self.bones.len()),
                s_morph_index: optimal_data_type(self.morphs.len()),
                s_rigid_body_index: optimal_data_type(self.rigid_bodies.len()),
            },
            ext_2_1,
        )
    }

    fn burn_by_writer<W: Write>(
        &self,
        mut writer: BinaryWriter<W>,
        ext_2_1: bool,
    ) -> Result<(), WritePMXErrors> {
        let model_info = if let Some(mi) = &self.model_info {
            mi
        } else {
            return Err(WritePMXErrors::NoModelInfo);
        };
        writer.write_header();
        writer.write_text_buf(&model_info.name);
        writer.write_text_buf(&model_info.name_en);
        writer.write_text_buf(&model_info.comment);
        writer.write_text_buf(&model_info.comment_en);
        //wrote model info

        writer.write_i32(i32::try_from(self.vertices.len())?);
        self.vertices
            .iter()
            .for_each(|vertex| writer.write_vertex(vertex));

        writer.write_i32(i32::try_from(3 * self.faces.len())?);
        self.faces.iter().for_each(|face| writer.write_face(face));

        writer.write_i32(i32::try_from(self.textures.len())?);
        self.textures
            .iter()
            .for_each(|name| writer.write_text_buf(name));

        writer.write_i32(i32::try_from(self.materials.len())?);
        self.materials
            .iter()
            .for_each(|material| writer.write_material(material));

        writer.write_i32(i32::try_from(self.bones.len())?);
        self.bones.iter().for_each(|bone| writer.write_bone(bone));

        writer.write_i32(i32::try_from(self.morphs.len())?);
        self.morphs
            .iter()
            .for_each(|morph| writer.write_morph(morph));

        writer.write_i32(i32::try_from(self.frames.len())?);
        self.frames
            .iter()
            .for_each(|frame| writer.write_frame(frame));

        writer.write_i32(i32::try_from(self.rigid_bodies.len())?);
        self.rigid_bodies
            .iter()
            .for_each(|rigid| writer.write_rigid(rigid));

        writer.write_i32(i32::try_from(self.joints.len())?);
        self.joints
            .iter()
            .for_each(|joint| writer.write_joint(joint));

        // 2.1 extended section.
        if ext_2_1 {
            writer.write_i32(i32::try_from(self.soft_bodies.len())?);
            self.soft_bodies
                .iter()
                .for_each(|soft_body| writer.write_soft_body(soft_body));
        }
        writer.inner.flush().map_err(WritePMXErrors::IoError)
    }

    /// write all data to file and drop it
    ///
    /// # Panics
    ///
    /// # Errors
    /// * `WritePMXErrors::TooBig` if any buffer elements exceeds `i32::MAX`
    /// * `WritePMXErrors::NoModelInfo` if model info is not set.
    /// * `WritePMXErrors::IoError` if failed to write pmx.
    pub fn write_to_path<P: AsRef<Path>>(self, path: P) -> Result<(), WritePMXErrors> {
        let (header, ext_2_1) = self.calculate_header();
        let writer = crate::binary_writer::BinaryWriter::create(path, header)?;
        self.burn_by_writer(writer, ext_2_1)
    }

    /// write all data to Stream and drop it
    ///
    /// # Panics
    ///
    /// # Errors
    /// * `WritePMXErrors::TooBig` if any buffer elements exceeds `i32::MAX`
    /// * `WritePMXErrors::NoModelInfo` if model info is not set.
    pub fn write<W: Write>(self, writer: W) -> Result<(), WritePMXErrors> {
        let (header, ext_2_1) = self.calculate_header();
        let writer = crate::binary_writer::BinaryWriter::from_writer(writer, header);
        self.burn_by_writer(writer, ext_2_1)
    }
}

fn optimal_data_type_vertex(len: usize) -> VertexIndexKinds {
    if u8::try_from(len).is_ok() {
        VertexIndexKinds::U8 //8 bit
    } else if u16::try_from(len).is_ok() {
        VertexIndexKinds::U16 //16 bit
    } else {
        VertexIndexKinds::I32 //32 bit
    }
}

fn optimal_data_type(len: usize) -> IndexKinds {
    if i8::try_from(len).is_ok() {
        IndexKinds::I8 //8 bit
    } else if i16::try_from(len).is_ok() {
        IndexKinds::I16 //16 bit
    } else {
        IndexKinds::I32 //32 bit
    }
}

#[derive(Debug)]
pub enum WritePMXErrors {
    NoModelInfo,
    IoError(std::io::Error),
    TooBig,
}

impl From<std::io::Error> for WritePMXErrors {
    fn from(err: Error) -> Self {
        Self::IoError(err)
    }
}
impl From<TryFromIntError> for WritePMXErrors {
    fn from(_: TryFromIntError) -> Self {
        Self::TooBig
    }
}
