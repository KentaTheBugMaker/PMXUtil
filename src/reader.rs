//! # PMX reading module.
//! this module separated to some parts.To avoid invalid reading.
//!
//! |Current stage|product|Next stage|
//! |-------------|-------|----------|
//! |[`ModelInfoStage`]|[`ModelInfo`]|[`VerticesStage`]|
//! |[`VerticesStage`]|[`Vec<Vertex>`]|[`FacesStage`]|
//! |[`FacesStage`]|[`Vec<Face>`]|[`TexturesStage`]|
//! |[`TexturesStage`]|[`TextureList`]|[`MaterialsStage`]|
//! |[`MaterialsStage`]|[`Vec<Material>`]|[`BonesStage`]|
//! |[`BonesStage`]|[`Vec<Bone>`]|[`MorphsStage`]|
//! |[`MorphsStage`]|[`Vec<Morph>`]|[`FrameStage`]|
//! |[`FrameStage`]|[`Vec<Frame>`]|[`RigidStage`]|
//! |[`RigidStage`]|[`Vec<Rigid>`]|[`JointStage`]|
//! |[`JointStage`]|[`Vec<Joint>`]|[`Option<SoftBodyStage>`]|
//! |[`SoftBodyStage`]|[`Vec<SoftBody>`]|There are no reader|
//! ```rust
//! // i want to get pmx path from env vars.
//! let path = std::env::var("PMX_FILE").unwrap();
//! let model_info_loader=PMXUtil::reader::ModelInfoStage::open(path);
//! let (model_info,vertices_loader)=model_info_loader.unwrap().read();
//! ```
//!

use crate::binary_reader::BinaryReader;
use crate::types::{
    Bone, BoneFlags, BoneIKInfo, BoneMorph, ConnectionDisplayMode, ControlPanel, Encode, Face,
    FlipMorph, Frame, FrameInner, GroupMorph, Header, HeaderConversionError, HeaderRaw, IKLink,
    ImpulseMorph, Joint, JointParameterRaw, JointType, Material, MaterialFlags, MaterialMorph,
    ModelInfo, Morph, MorphKinds, PMXVersion, Rigid, RigidCalcMethod, RigidForm,
    RotateAndTranslateInherits, SoftBody, SoftBodyAeroModel, SoftBodyAnchorRigid, SoftBodyForm,
    SphereMode, SphereModeKind, Target, TextureList, ToonMode, UVMorph, Vertex, VertexMorph,
    VertexWeight,
};
use std::convert::TryInto;
use std::path::Path;

fn transform_header_c2r(header: &HeaderRaw) -> Result<Header, HeaderConversionError> {
    if header.magic == [0x50, 0x4d, 0x58, 0x20] {
        Ok(Header {
            magic: "PMX ".to_owned(),
            version: if header.version >= 2.0 {
                if header.version < 2.2 {
                    if header.version > 2.05 {
                        PMXVersion::V21
                    } else {
                        PMXVersion::V20
                    }
                } else {
                    return Err(HeaderConversionError::InvalidVersion);
                }
            } else {
                return Err(HeaderConversionError::InvalidVersion);
            },
            length: header.length,
            encode: match header.config[0] {
                0 => Encode::Utf16Le,
                1 => Encode::UTF8,
                _ => {
                    return Err(HeaderConversionError::InvalidEncoding);
                }
            },
            additional_uv: header.config[1],
            s_vertex_index: header.config[2].try_into().unwrap(),
            s_texture_index: header.config[3].try_into().unwrap(),
            s_material_index: header.config[4].try_into().unwrap(),
            s_bone_index: header.config[5].try_into().unwrap(),
            s_morph_index: header.config[6].try_into().unwrap(),
            s_rigid_body_index: header.config[7].try_into().unwrap(),
        })
    } else {
        Err(HeaderConversionError::InvalidMagic)
    }
}

pub struct ModelInfoStage(ReaderInner);

impl ModelInfoStage {
    /// the start of reader module.
    /// # None
    /// * invalid path given
    /// * read  magic number is not ` `
    /// # Arguments
    ///
    /// * `path`: path to pmx file.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// let path = std::env::var("PMX_FILE").unwrap();
    /// let model_info_loader = PMXUtil::reader::ModelInfoStage::open(path).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Option<ModelInfoStage> {
        let mut inner = BinaryReader::open(path).ok()?;
        let header = inner.read_raw_header();
        let header_rs = transform_header_c2r(&header).ok()?;
        Some(ModelInfoStage(ReaderInner {
            inner,
            header: header_rs,
        }))
    }

    pub fn get_header(&self) -> Header {
        self.0.header.clone()
    }

    pub fn read(mut self) -> (ModelInfo, VerticesStage) {
        (
            ModelInfo {
                name: self.0.read_text_buf(),
                name_en: self.0.read_text_buf(),
                comment: self.0.read_text_buf(),
                comment_en: self.0.read_text_buf(),
            },
            VerticesStage(self.0),
        )
    }
}
pub struct VerticesStage(ReaderInner);
impl VerticesStage {
    pub fn read(mut self) -> (Vec<Vertex>, FacesStage) {
        (
            (0..self.0.read_i32())
                .map(|_| self.read_pmx_vertex())
                .collect(),
            FacesStage(self.0),
        )
    }

    fn read_pmx_vertex(&mut self) -> Vertex {
        let mut ctx = Vertex {
            position: [0.0; 3],
            norm: [0.0; 3],
            uv: [0.0; 2],
            add_uv: [[0.0; 4]; 4],
            weight_type: VertexWeight::BDEF1(-1),
            edge_mag: 0.0,
        };
        ctx.position = self.0.read_vec3();
        ctx.norm = self.0.read_vec3();
        ctx.uv = self.0.read_vec2();
        let additional_uv = self.0.header.additional_uv as usize;
        if additional_uv > 0 {
            for i in 0..additional_uv {
                ctx.add_uv[i] = self.0.read_vec4();
            }
        }
        let weight_type = self.0.read_u8();
        ctx.weight_type = match weight_type {
            0 => {
                let index = self.0.read_bone_index();
                VertexWeight::BDEF1(index)
            }

            1 => {
                let bone_index_1 = self.0.read_bone_index();
                let bone_index_2 = self.0.read_bone_index();
                let bone_weight_1 = self.0.read_f32();
                VertexWeight::BDEF2 {
                    bone_index_1,
                    bone_index_2,
                    bone_weight_1,
                }
            }
            2 | 4 => {
                let bone_index_1 = self.0.read_bone_index();
                let bone_index_2 = self.0.read_bone_index();
                let bone_index_3 = self.0.read_bone_index();
                let bone_index_4 = self.0.read_bone_index();
                let bone_weight_1 = self.0.read_f32();
                let bone_weight_2 = self.0.read_f32();
                let bone_weight_3 = self.0.read_f32();
                let bone_weight_4 = self.0.read_f32();
                if weight_type == 2 {
                    VertexWeight::BDEF4 {
                        bone_index_1,
                        bone_index_2,
                        bone_index_3,
                        bone_index_4,
                        bone_weight_1,
                        bone_weight_2,
                        bone_weight_3,
                        bone_weight_4,
                    }
                } else {
                    VertexWeight::QDEF {
                        bone_index_1,
                        bone_index_2,
                        bone_index_3,
                        bone_index_4,
                        bone_weight_1,
                        bone_weight_2,
                        bone_weight_3,
                        bone_weight_4,
                    }
                }
            }
            3 => {
                let bone_index_1 = self.0.read_bone_index();
                let bone_index_2 = self.0.read_bone_index();
                let bone_weight_1 = self.0.read_f32();
                let sdef_c = self.0.read_vec3();
                let sdef_r0 = self.0.read_vec3();
                let sdef_r1 = self.0.read_vec3();
                VertexWeight::SDEF {
                    bone_index_1,
                    bone_index_2,
                    bone_weight_1,
                    sdef_c,
                    sdef_r0,
                    sdef_r1,
                }
            }
            _ => {
                panic!("Unknown Weight type:{}", weight_type);
            }
        };

        ctx.edge_mag = self.0.read_f32();
        ctx
    }
}
pub struct FacesStage(ReaderInner);
impl FacesStage {
    /// Read the faces
    ///
    /// read [Face doc](crate::types::Face)
    pub fn read(mut self) -> (Vec<Face>, TexturesStage) {
        (
            (0..(self.0.read_i32() / 3))
                .map(|_| Face {
                    vertices: [
                        self.0.read_vertex_index(),
                        self.0.read_vertex_index(),
                        self.0.read_vertex_index(),
                    ],
                })
                .collect(),
            TexturesStage(self.0),
        )
    }
}

pub struct TexturesStage(ReaderInner);
impl TexturesStage {
    /// Read relative texture path from current reading file
    ///
    /// # Note
    /// for Unix like -system user you need to convert \ to /
    pub fn read(mut self) -> (TextureList, MaterialsStage) {
        (
            TextureList {
                textures: (0..self.0.read_i32())
                    .map(|_| self.0.read_text_buf())
                    .collect(),
            },
            MaterialsStage(self.0),
        )
    }
}
pub struct MaterialsStage(ReaderInner);
impl MaterialsStage {
    ///Read material's information contains name ambient diffuse specular etc parameters.
    ///
    /// please read [Material](crate::types::Material) doc
    pub fn read(mut self) -> (Vec<Material>, BonesStage) {
        (
            (0..self.0.read_i32())
                .map(|_| self.read_pmx_material())
                .collect(),
            BonesStage(self.0),
        )
    }

    fn read_pmx_material(&mut self) -> Material {
        Material {
            name: self.0.read_text_buf(),
            english_name: self.0.read_text_buf(),
            diffuse: self.0.read_vec4(),
            specular: self.0.read_vec3(),
            specular_factor: self.0.read_f32(),
            ambient: self.0.read_vec3(),
            draw_mode: MaterialFlags::from_bits_truncate(self.0.read_u8()),
            edge_color: self.0.read_vec4(),
            edge_size: self.0.read_f32(),
            texture_index: self.0.read_texture_index(),
            sphere_mode: {
                let ti = self.0.read_texture_index();
                match self.0.read_u8() {
                    0 => None,
                    1 => Some(SphereMode {
                        kind: SphereModeKind::Mul,
                        index: ti,
                    }),
                    2 => Some(SphereMode {
                        kind: SphereModeKind::Add,
                        index: ti,
                    }),
                    3 => Some(SphereMode {
                        kind: SphereModeKind::SubTexture,
                        index: ti,
                    }),
                    _ => {
                        panic!("Invalid sphere mode detected in material")
                    }
                }
            },
            toon_mode: match self.0.read_u8() {
                0 => ToonMode::Separate(self.0.read_texture_index()),
                1 => ToonMode::Common(self.0.read_u8()),
                _ => {
                    panic!("Invalid toon mode detected in material")
                }
            },
            memo: self.0.read_text_buf(),
            num_face_vertices: self.0.read_i32(),
        }
    }
}
pub struct BonesStage(ReaderInner);
impl BonesStage {
    /// read bone's information parent child IK etc.
    /// Exact model pose you should process this parameter
    pub fn read(mut self) -> (Vec<Bone>, MorphsStage) {
        (
            (0..self.0.read_i32())
                .map(|_| self.read_pmx_bone())
                .collect(),
            MorphsStage(self.0),
        )
    }
    fn read_pmx_bone(&mut self) -> Bone {
        let mut ctx = Bone {
            name: self.0.read_text_buf(),
            english_name: self.0.read_text_buf(),
            position: self.0.read_vec3(),
            parent: self.0.read_bone_index(),
            deform_depth: self.0.read_i32(),
            ..crate::types::Bone::default()
        };
        let bone_flags = BoneFlags::from_bits_truncate(self.0.read_u16());
        ctx.controllable_in_viewer = bone_flags.intersects(BoneFlags::ENABLED);
        ctx.display_bone_in_viewer = bone_flags.intersects(BoneFlags::IS_VISIBLE);
        ctx.rotatable_in_viewer = bone_flags.intersects(BoneFlags::ROTATABLE);
        ctx.translatable_in_viewer = bone_flags.intersects(BoneFlags::TRANSLATABLE);
        if bone_flags.intersects(BoneFlags::CONNECT_TO_OTHER_BONE) {
            ctx.connection_display_mode =
                ConnectionDisplayMode::OtherBone(self.0.read_bone_index());
        } else {
            ctx.connection_display_mode = ConnectionDisplayMode::Offset(self.0.read_vec3());
        }
        ctx.inherits.inherit_local = bone_flags.intersects(BoneFlags::INHERIT_LOCAL);
        ctx.inherits.rotate_and_translate = match (
            bone_flags.intersects(BoneFlags::INHERIT_ROTATION),
            bone_flags.intersects(BoneFlags::INHERIT_TRANSLATION),
        ) {
            (true, true) => {
                RotateAndTranslateInherits::Both(self.0.read_bone_index(), self.0.read_f32())
            }
            (false, false) => RotateAndTranslateInherits::None,
            (true, false) => {
                RotateAndTranslateInherits::Rotate(self.0.read_bone_index(), self.0.read_f32())
            }
            (false, true) => {
                RotateAndTranslateInherits::Translate(self.0.read_bone_index(), self.0.read_f32())
            }
        };

        if bone_flags.intersects(BoneFlags::FIXED_AXIS) {
            ctx.fixed_axis = Some(self.0.read_vec3());
        }
        if bone_flags.intersects(BoneFlags::LOCAL_COORDINATE) {
            ctx.local_axis = Some((self.0.read_vec3(), self.0.read_vec3()));
        }
        if bone_flags.intersects(BoneFlags::EXTERNAL_PARENT_DEFORM) {
            ctx.external_parent = Some(self.0.read_i32());
        }
        if bone_flags.intersects(BoneFlags::IK) {
            ctx.ik_info = Some(BoneIKInfo {
                ik_target_bone_index: self.0.read_bone_index(),
                ik_iter_count: self.0.read_i32(),
                ik_limit_angle: self.0.read_f32(),
                ik_links: (0..self.0.read_i32()).map(|_| self.read_iklink()).collect(),
            });
        }
        ctx
    }
    fn read_iklink(&mut self) -> IKLink {
        IKLink {
            ik_bone_index: self.0.read_bone_index(),
            angle_limit: match self.0.read_u8() {
                0 => None,
                1 => Some((self.0.read_vec3(), self.0.read_vec3())),
                x => {
                    panic!("we cant determine angle limit enabled because {}", x)
                }
            },
        }
    }
}

pub struct MorphsStage(ReaderInner);
impl MorphsStage {
    pub fn read(mut self) -> (Vec<Morph>, FrameStage) {
        (
            (0..self.0.read_i32())
                .map(|_| self.read_pmx_morph())
                .collect(),
            FrameStage(self.0),
        )
    }

    fn read_pmx_morph(&mut self) -> Morph {
        Morph {
            name: self.0.read_text_buf(),
            english_name: self.0.read_text_buf(),
            control_panel: match self.0.read_u8() {
                0 => ControlPanel::System,
                1 => ControlPanel::BottomLeft,
                2 => ControlPanel::TopLeft,
                3 => ControlPanel::TopRight,
                4 => ControlPanel::BottomRight,
                x => panic!("Detected unknown morph control panel {} ", x),
            },
            morph_data: {
                let morph_kind = self.0.read_u8();
                match morph_kind {
                    0 => MorphKinds::Group(
                        (0..self.0.read_i32())
                            .map(|_| self.read_group_morph())
                            .collect(),
                    ),
                    1 => MorphKinds::Vertex(
                        (0..self.0.read_i32())
                            .map(|_| self.read_vertex_morph())
                            .collect(),
                    ),
                    2 => MorphKinds::Bone(
                        (0..self.0.read_i32())
                            .map(|_| self.read_bone_morph())
                            .collect(),
                    ),
                    3 => MorphKinds::UV(
                        (0..self.0.read_i32())
                            .map(|_| self.read_uv_morph())
                            .collect(),
                    ),
                    4 => MorphKinds::UV1(
                        (0..self.0.read_i32())
                            .map(|_| self.read_uv_morph())
                            .collect(),
                    ),
                    5 => MorphKinds::UV2(
                        (0..self.0.read_i32())
                            .map(|_| self.read_uv_morph())
                            .collect(),
                    ),
                    6 => MorphKinds::UV3(
                        (0..self.0.read_i32())
                            .map(|_| self.read_uv_morph())
                            .collect(),
                    ),
                    7 => MorphKinds::UV4(
                        (0..self.0.read_i32())
                            .map(|_| self.read_uv_morph())
                            .collect(),
                    ),
                    8 => MorphKinds::Material(
                        (0..self.0.read_i32())
                            .map(|_| self.read_material_morph())
                            .collect(),
                    ),
                    9 => MorphKinds::Flip(
                        (0..self.0.read_i32())
                            .map(|_| self.read_flip_morph())
                            .collect(),
                    ),
                    10 => MorphKinds::Impulse(
                        (0..self.0.read_i32())
                            .map(|_| self.read_impulse_morph())
                            .collect(),
                    ),
                    x => panic!("Unknown morph kind {} detected.", x),
                }
            },
        }
    }
    fn read_vertex_morph(&mut self) -> VertexMorph {
        VertexMorph {
            index: self.0.read_vertex_index(),
            offset: self.0.read_vec3(),
        }
    }
    fn read_uv_morph(&mut self) -> UVMorph {
        UVMorph {
            index: self.0.read_vertex_index(),
            offset: self.0.read_vec4(),
        }
    }
    fn read_bone_morph(&mut self) -> BoneMorph {
        BoneMorph {
            index: self.0.read_bone_index(),
            translates: self.0.read_vec3(),
            rotates: self.0.read_vec4(),
        }
    }
    fn read_material_morph(&mut self) -> MaterialMorph {
        MaterialMorph {
            index: self.0.read_material_index(),
            formula: self.0.read_u8(),
            diffuse: self.0.read_vec4(),
            specular: self.0.read_vec3(),
            specular_factor: self.0.read_f32(),
            ambient: self.0.read_vec3(),
            edge_color: self.0.read_vec4(),
            edge_size: self.0.read_f32(),
            texture_factor: self.0.read_vec4(),
            sphere_texture_factor: self.0.read_vec4(),
            toon_texture_factor: self.0.read_vec4(),
        }
    }
    fn read_group_morph(&mut self) -> GroupMorph {
        GroupMorph {
            index: self.0.read_morph_index(),
            morph_factor: self.0.read_f32(),
        }
    }
    fn read_flip_morph(&mut self) -> FlipMorph {
        FlipMorph {
            index: self.0.read_morph_index(),
            morph_factor: self.0.read_f32(),
        }
    }
    fn read_impulse_morph(&mut self) -> ImpulseMorph {
        ImpulseMorph {
            rigid_index: self.0.read_rigid_index(),
            is_local: self.0.read_u8(),
            velocity: self.0.read_vec3(),
            torque: self.0.read_vec3(),
        }
    }
}
pub struct FrameStage(ReaderInner);

impl FrameStage {
    /// read `MMD` controller
    /// # Panics
    /// * if contains invalid target
    pub fn read(mut self) -> (Vec<Frame>, RigidStage) {
        (
            (0..self.0.read_i32())
                .map(|_| Frame {
                    name: self.0.read_text_buf(),
                    name_en: self.0.read_text_buf(),
                    is_special: self.0.read_u8(),
                    inners: (0..self.0.read_i32())
                        .map(|_| {
                            let target = self.0.read_u8();
                            match target {
                                0 => FrameInner {
                                    target: Target::Bone,
                                    index: self.0.read_bone_index(),
                                },
                                1 => FrameInner {
                                    target: Target::Morph,
                                    index: self.0.read_morph_index(),
                                },
                                x => {
                                    panic!("Invalid frame target detected {}", x)
                                }
                            }
                        })
                        .collect(),
                })
                .collect(),
            RigidStage(self.0),
        )
    }
}
pub struct RigidStage(ReaderInner);
impl RigidStage {
    pub fn read(mut self) -> (Vec<Rigid>, JointStage) {
        (
            (0..self.0.read_i32())
                .map(|_| {
                    let name = self.0.read_text_buf();
                    let name_en = self.0.read_text_buf();
                    let bone_index = self.0.read_bone_index();
                    let group = self.0.read_u8();
                    let un_collision_group_flag = self.0.read_u16();
                    let form = match self.0.read_u8() {
                        0 => RigidForm::Sphere,
                        1 => RigidForm::Box,
                        2 => RigidForm::Capsule,
                        _ => {
                            unreachable!("Invalid  file detected at rigid loader")
                        }
                    };
                    let size = self.0.read_vec3();
                    let position = self.0.read_vec3();
                    let rotation = self.0.read_vec3();
                    let mass = self.0.read_f32();
                    let move_resist = self.0.read_f32();
                    let rotation_resist = self.0.read_f32();
                    let repulsion = self.0.read_f32();
                    let friction = self.0.read_f32();
                    let calc_method = match self.0.read_u8() {
                        0 => RigidCalcMethod::Static,
                        1 => RigidCalcMethod::Dynamic,
                        2 => RigidCalcMethod::DynamicWithBonePosition,
                        _ => {
                            unreachable!("Invalid  file detected as rigid loader")
                        }
                    };
                    Rigid {
                        name,
                        name_en,
                        bone_index,
                        group,
                        un_collision_group_flag,
                        form,
                        size,
                        position,
                        rotation,
                        mass,
                        move_resist,
                        rotation_resist,
                        repulsion,
                        friction,
                        calc_method,
                    }
                })
                .collect(),
            JointStage(self.0),
        )
    }
}

pub struct JointStage(ReaderInner);

impl JointStage {
    pub fn read(mut self) -> (Vec<Joint>, Option<SoftBodyStage>) {
        (
            (0..self.0.read_i32()).map(|_| self.read_joint()).collect(),
            if let crate::types::PMXVersion::V21 = self.0.header.version {
                //this file contains softbody section
                Some(SoftBodyStage(self.0))
            } else {
                None
            },
        )
    }
    fn read_joint(&mut self) -> Joint {
        let name = self.0.read_text_buf();
        let name_en = self.0.read_text_buf();
        let raw_parameter = {
            JointParameterRaw {
                joint_type: self.0.read_u8(),
                a_rigid_index: self.0.read_rigid_index(),
                b_rigid_index: self.0.read_rigid_index(),
                position: self.0.read_vec3(),
                rotation: self.0.read_vec3(),
                move_limit_down: self.0.read_vec3(),
                move_limit_up: self.0.read_vec3(),
                rotation_limit_down: self.0.read_vec3(),
                rotation_limit_up: self.0.read_vec3(),
                spring_const_move: self.0.read_vec3(),
                spring_const_rotation: self.0.read_vec3(),
            }
        };

        let joint_parameter = translate_joint(&raw_parameter);
        Joint {
            name,
            name_en,
            joint_type: joint_parameter,
        }
    }
}
fn translate_joint(raw_parameter: &JointParameterRaw) -> JointType {
    fn is_eq_f32(lhs: f32, rhs: f32) -> bool {
        (lhs - rhs).abs() < 0.01
    }
    match raw_parameter.joint_type {
        0 => JointType::Spring6DOF {
            a_rigid_index: raw_parameter.a_rigid_index,
            b_rigid_index: raw_parameter.b_rigid_index,
            position: raw_parameter.position,
            rotation: raw_parameter.rotation,
            move_limit_down: raw_parameter.move_limit_down,
            move_limit_up: raw_parameter.move_limit_up,
            rotation_limit_down: raw_parameter.rotation_limit_down,
            rotation_limit_up: raw_parameter.rotation_limit_up,
            spring_const_move: raw_parameter.spring_const_move,
            spring_const_rotation: raw_parameter.spring_const_rotation,
        },
        1 => JointType::SixDof {
            a_rigid_index: raw_parameter.a_rigid_index,
            b_rigid_index: raw_parameter.b_rigid_index,
            position: raw_parameter.position,
            rotation: raw_parameter.rotation,
            move_limit_down: raw_parameter.move_limit_down,
            move_limit_up: raw_parameter.move_limit_up,
            rotation_limit_down: raw_parameter.rotation_limit_down,
            rotation_limit_up: raw_parameter.rotation_limit_up,
        },
        2 => JointType::P2P {
            a_rigid_index: raw_parameter.a_rigid_index,
            b_rigid_index: raw_parameter.b_rigid_index,
            position: raw_parameter.position,
            rotation: raw_parameter.rotation,
        },
        3 => JointType::ConeTwist {
            a_rigid_index: raw_parameter.a_rigid_index,
            b_rigid_index: raw_parameter.b_rigid_index,
            swing_span1: raw_parameter.rotation_limit_down[2],
            swing_span2: raw_parameter.rotation_limit_down[1],
            twist_span: raw_parameter.rotation_limit_down[0],
            softness: raw_parameter.spring_const_move[0],
            bias_factor: raw_parameter.spring_const_move[1],
            relaxation_factor: raw_parameter.spring_const_move[2],
            damping: raw_parameter.move_limit_down[0],
            fix_thresh: raw_parameter.move_limit_up[0],
            enable_motor: is_eq_f32(raw_parameter.move_limit_down[2], 1.0),
            max_motor_impulse: raw_parameter.move_limit_up[2],
            motor_target_in_constraint_space: raw_parameter.spring_const_rotation,
        },
        4 => JointType::Slider {
            a_rigid_index: raw_parameter.a_rigid_index,
            b_rigid_index: raw_parameter.b_rigid_index,
            lower_linear_limit: raw_parameter.move_limit_down[0],
            upper_linear_limit: raw_parameter.move_limit_up[0],
            lower_angle_limit: raw_parameter.rotation_limit_down[0],
            upper_angle_limit: raw_parameter.rotation_limit_up[0],
            power_linear_motor: is_eq_f32(raw_parameter.spring_const_move[0], 1.0),
            target_linear_motor_velocity: raw_parameter.spring_const_move[1],
            max_linear_motor_force: raw_parameter.spring_const_move[2],
            power_angler_motor: is_eq_f32(raw_parameter.spring_const_rotation[0], 1.0),
            target_angler_motor_velocity: raw_parameter.spring_const_rotation[1],
            max_angler_motor_force: raw_parameter.spring_const_rotation[2],
        },
        5 => JointType::Hinge {
            a_rigid_index: raw_parameter.a_rigid_index,
            b_rigid_index: raw_parameter.b_rigid_index,
            low: raw_parameter.move_limit_down[0],
            high: raw_parameter.move_limit_up[0],
            softness: raw_parameter.spring_const_move[0],
            bias_factor: raw_parameter.spring_const_move[1],
            relaxation_factor: raw_parameter.spring_const_move[2],
            enable_motor: is_eq_f32(raw_parameter.spring_const_rotation[0], 1.0),
            target_velocity: raw_parameter.spring_const_rotation[1],
            max_motor_impulse: raw_parameter.spring_const_rotation[2],
        },
        _ => {
            unreachable!("Invalid joint type detected in joint loader")
        }
    }
}
pub struct SoftBodyStage(ReaderInner);

impl SoftBodyStage {
    pub fn read(mut self) -> Vec<SoftBody> {
        (0..self.0.read_i32())
            .map(|_| self.read_soft_body())
            .collect()
    }
    fn read_soft_body(&mut self) -> SoftBody {
        SoftBody {
            name: self.0.read_text_buf(),
            name_en: self.0.read_text_buf(),
            form: match self.0.read_u8() {
                0 => SoftBodyForm::TriMesh,
                1 => SoftBodyForm::Rope,
                _ => {
                    panic!("Error invalid SoftBodyForm ")
                }
            },
            material_index: self.0.read_material_index(),
            group: self.0.read_u8(),
            un_collision_group_flag: self.0.read_u16(),
            bit_flag: self.0.read_u8(),
            b_link_create_distance: self.0.read_i32(),
            clusters: self.0.read_i32(),
            mass: self.0.read_f32(),
            collision_margin: self.0.read_f32(),
            aero_model: match self.0.read_i32() {
                0 => SoftBodyAeroModel::VPoint,
                1 => SoftBodyAeroModel::VTwoSide,
                2 => SoftBodyAeroModel::VOneSided,
                3 => SoftBodyAeroModel::FTwoSided,
                4 => SoftBodyAeroModel::FOneSided,
                _ => {
                    panic!("Error invalid SoftBodyAeroModel")
                }
            },
            //config
            vcf: self.0.read_f32(),
            dp: self.0.read_f32(),
            dg: self.0.read_f32(),
            lf: self.0.read_f32(),
            pr: self.0.read_f32(),
            vc: self.0.read_f32(),
            df: self.0.read_f32(),
            mt: self.0.read_f32(),
            chr: self.0.read_f32(),
            khr: self.0.read_f32(),
            shr: self.0.read_f32(),
            ahr: self.0.read_f32(),
            //cluster
            srhr_cl: self.0.read_f32(),
            skhr_cl: self.0.read_f32(),
            sshr_cl: self.0.read_f32(),
            sr_splt_cl: self.0.read_f32(),
            sk_splt_cl: self.0.read_f32(),
            ss_splt_cl: self.0.read_f32(),
            //iteration
            v_it: self.0.read_i32(),
            p_it: self.0.read_i32(),
            d_it: self.0.read_i32(),
            c_it: self.0.read_i32(),
            //material
            lst: self.0.read_f32(),
            ast: self.0.read_f32(),
            vst: self.0.read_f32(),
            anchor_rigid: (0..self.0.read_i32())
                .map(|_| SoftBodyAnchorRigid {
                    rigid_index: self.0.read_rigid_index(),
                    vertex_index: self.0.read_vertex_index(),
                    near_mode: match self.0.read_i8() {
                        0 => true,
                        1 => false,
                        x => {
                            panic!("invalid near mode {}", x)
                        }
                    },
                })
                .collect(),
            pin_vertex: (0..self.0.read_i32())
                .map(|_| self.0.read_vertex_index())
                .collect(),
        }
    }
}
struct ReaderInner {
    inner: BinaryReader,
    header: Header,
}

impl ReaderInner {
    pub fn read_vertex_index(&mut self) -> i32 {
        self.inner.read_vertex_index(self.header.s_vertex_index)
    }

    pub fn read_texture_index(&mut self) -> i32 {
        self.inner.read_sized(self.header.s_texture_index)
    }

    pub fn read_material_index(&mut self) -> i32 {
        self.inner.read_sized(self.header.s_material_index)
    }

    pub fn read_bone_index(&mut self) -> i32 {
        self.inner.read_sized(self.header.s_bone_index)
    }

    pub fn read_morph_index(&mut self) -> i32 {
        self.inner.read_sized(self.header.s_morph_index)
    }

    pub fn read_rigid_index(&mut self) -> i32 {
        self.inner.read_sized(self.header.s_rigid_body_index)
    }

    pub fn read_u8(&mut self) -> u8 {
        self.inner.read_u8()
    }

    pub fn read_u16(&mut self) -> u16 {
        self.inner.read_u16()
    }

    pub fn read_i8(&mut self) -> i8 {
        self.inner.read_i8()
    }

    pub fn read_i32(&mut self) -> i32 {
        self.inner.read_i32()
    }

    pub fn read_vec4(&mut self) -> [f32; 4] {
        self.inner.read_vec4()
    }

    pub fn read_vec3(&mut self) -> [f32; 3] {
        self.inner.read_vec3()
    }

    pub fn read_vec2(&mut self) -> [f32; 2] {
        self.inner.read_vec2()
    }

    pub fn read_f32(&mut self) -> f32 {
        self.inner.read_f32()
    }

    pub fn read_text_buf(&mut self) -> String {
        self.inner.read_text_buf(self.header.encode)
    }
}
