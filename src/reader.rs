//! #  reading module.
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
//! let model_info_loader=pmx_util::reader::ModelInfoStage::open(path);
//! let (model_info,vertices_loader)=model_info_loader.unwrap().read();
//! ```
//!

use crate::binary_reader::BinaryReader;
use crate::types::{
    Bone, BoneFlags, BoneMorph, Encode, Face, Frame, FrameInner, GroupMorph, Header,
    HeaderConversionError, HeaderRaw, IKLink, Joint, JointParameterRaw, JointType, Material,
    MaterialFlags, MaterialMorph, ModelInfo, Morph, MorphKinds, Rigid, RigidCalcMethod, RigidForm,
    SoftBody, SoftBodyAeroModel, SoftBodyAnchorRigid, SoftBodyForm, SphereMode, SphereModeKind,
    TextureList, ToonMode, UVMorph, Vertex, VertexMorph, VertexWeight,
};
use std::convert::TryInto;
use std::path::Path;

fn transform_header_c2r(header: &HeaderRaw) -> Result<Header, HeaderConversionError> {
    if header.magic == [0x50, 0x4d, 0x58, 0x20] {
        Ok(Header {
            magic: "PMX ".to_owned(),
            version: header.version,
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

pub struct ModelInfoStage {
    header: Header,
    inner: BinaryReader,
}
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
    /// let model_info_loader = pmx_util::reader::ModelInfoStage::open(path).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Option<ModelInfoStage> {
        let mut inner = BinaryReader::open(path).ok()?;
        let header = inner.read_raw_header();
        let header_rs = transform_header_c2r(&header).ok()?;
        Some(ModelInfoStage {
            header: header_rs,
            inner,
        })
    }

    pub fn get_header(&self) -> Header {
        self.header.clone()
    }

    pub fn read(mut self) -> (ModelInfo, VerticesStage) {
        let enc = self.header.encode;
        (
            ModelInfo {
                name: self.inner.read_text_buf(enc),
                name_en: self.inner.read_text_buf(enc),
                comment: self.inner.read_text_buf(enc),
                comment_en: self.inner.read_text_buf(enc),
            },
            VerticesStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}
pub struct VerticesStage {
    header: Header,
    inner: BinaryReader,
}
impl VerticesStage {
    pub fn read(mut self) -> (Vec<Vertex>, FacesStage) {
        (
            (0..self.inner.read_i32())
                .map(|_| self.read_pmx_vertex())
                .collect(),
            FacesStage {
                header: self.header,
                inner: self.inner,
            },
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
        ctx.position = self.inner.read_vec3();
        ctx.norm = self.inner.read_vec3();
        ctx.uv = self.inner.read_vec2();
        let additional_uv = self.header.additional_uv as usize;
        let size = self.header.s_bone_index;
        if additional_uv > 0 {
            for i in 0..additional_uv {
                ctx.add_uv[i] = self.inner.read_vec4();
            }
        }
        let weight_type = self.inner.read_u8();
        ctx.weight_type = match weight_type {
            0 => {
                let index = self.inner.read_sized(size);
                VertexWeight::BDEF1(index)
            }

            1 => {
                let bone_index_1 = self.inner.read_sized(size);
                let bone_index_2 = self.inner.read_sized(size);
                let bone_weight_1 = self.inner.read_f32();
                VertexWeight::BDEF2 {
                    bone_index_1,
                    bone_index_2,
                    bone_weight_1,
                }
            }
            2 | 4 => {
                let bone_index_1 = self.inner.read_sized(size);
                let bone_index_2 = self.inner.read_sized(size);
                let bone_index_3 = self.inner.read_sized(size);
                let bone_index_4 = self.inner.read_sized(size);
                let bone_weight_1 = self.inner.read_f32();
                let bone_weight_2 = self.inner.read_f32();
                let bone_weight_3 = self.inner.read_f32();
                let bone_weight_4 = self.inner.read_f32();
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
                let bone_index_1 = self.inner.read_sized(size);
                let bone_index_2 = self.inner.read_sized(size);
                let bone_weight_1 = self.inner.read_f32();
                let sdef_c = self.inner.read_vec3();
                let sdef_r0 = self.inner.read_vec3();
                let sdef_r1 = self.inner.read_vec3();
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

        ctx.edge_mag = self.inner.read_f32();
        ctx
    }
}
pub struct FacesStage {
    header: Header,
    inner: BinaryReader,
}
impl FacesStage {
    /// Read the faces
    ///
    /// read [Face doc](crate::types::Face)
    pub fn read(mut self) -> (Vec<Face>, TexturesStage) {
        let s_vertex_index = self.header.s_vertex_index;
        (
            (0..(self.inner.read_i32() / 3))
                .map(|_| Face {
                    vertices: [
                        self.inner.read_vertex_index(s_vertex_index),
                        self.inner.read_vertex_index(s_vertex_index),
                        self.inner.read_vertex_index(s_vertex_index),
                    ],
                })
                .collect(),
            TexturesStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}

pub struct TexturesStage {
    header: Header,
    inner: BinaryReader,
}
impl TexturesStage {
    /// Read relative texture path from current reading file
    ///
    /// # Note
    /// for Unix like -system user you need to convert \ to /
    pub fn read(mut self) -> (TextureList, MaterialsStage) {
        (
            TextureList {
                textures: (0..self.inner.read_i32())
                    .into_iter()
                    .fold(vec![], |mut textures, _| {
                        textures.push(self.inner.read_text_buf(self.header.encode));
                        textures
                    }),
            },
            MaterialsStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}
pub struct MaterialsStage {
    header: Header,
    inner: BinaryReader,
}
impl MaterialsStage {
    ///Read material's information contains name ambient diffuse specular etc parameters.
    ///
    /// please read [Material](crate::types::Material) doc
    pub fn read(mut self) -> (Vec<Material>, BonesStage) {
        (
            (0..self.inner.read_i32()).fold(vec![], |mut materials, _| {
                materials.push(self.read_pmx_material());
                materials
            }),
            BonesStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }

    fn read_pmx_material(&mut self) -> Material {
        let s_texture_index = self.header.s_texture_index;
        let mut ctx = Material {
            name: "".to_string(),
            english_name: "".to_string(),
            diffuse: [0.0; 4],
            specular: [0.0; 3],
            specular_factor: 0.0,
            ambient: [0.0; 3],
            draw_mode: MaterialFlags::from_bits_truncate(0),
            edge_color: [0.0; 4],
            edge_size: 0.0,
            texture_index: 0,
            sphere_mode: None,
            toon_mode: ToonMode::Common(0),
            memo: "".to_string(),
            num_face_vertices: 0,
        };
        ctx.name = self.inner.read_text_buf(self.header.encode);
        ctx.english_name = self.inner.read_text_buf(self.header.encode);
        ctx.diffuse = self.inner.read_vec4();
        ctx.specular = self.inner.read_vec3();
        ctx.specular_factor = self.inner.read_f32();
        ctx.ambient = self.inner.read_vec3();
        ctx.draw_mode = MaterialFlags::from_bits_truncate(self.inner.read_u8());
        ctx.edge_color = self.inner.read_vec4();
        ctx.edge_size = self.inner.read_f32();
        ctx.texture_index = self.inner.read_sized(s_texture_index);
        let ti = self.inner.read_sized(s_texture_index);
        let spmode = self.inner.read_u8();
        ctx.sphere_mode = match spmode {
            0 => None,
            1 => Some(SphereMode {
                index: ti,
                kind: SphereModeKind::Mul,
            }),
            2 => Some(SphereMode {
                index: ti,
                kind: SphereModeKind::Add,
            }),
            3 => Some(SphereMode {
                index: ti,
                kind: SphereModeKind::SubTexture,
            }),
            _ => {
                panic!("Error Unknown SphereMode:{}", spmode);
            }
        };

        let toonmode = self.inner.read_u8();
        ctx.toon_mode = match toonmode {
            0 => ToonMode::Separate(self.inner.read_sized(s_texture_index)),
            1 => ToonMode::Common(self.inner.read_u8()),
            _ => panic!("Error Unknown Toon flag:{}", toonmode),
        };
        ctx.memo = self.inner.read_text_buf(self.header.encode);
        ctx.num_face_vertices = self.inner.read_i32();
        ctx
    }
}
pub struct BonesStage {
    header: Header,
    inner: BinaryReader,
}
impl BonesStage {
    /// read bone's information parent child IK etc.
    /// Exact model pose you should process this parameter
    pub fn read(mut self) -> (Vec<Bone>, MorphsStage) {
        (
            (0..self.inner.read_i32())
                .map(|_| self.read_pmx_bone())
                .collect(),
            MorphsStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }
    fn read_pmx_bone(&mut self) -> Bone {
        let encode = self.header.encode;
        let s_bone_index = self.header.s_bone_index;
        let mut ctx = Bone {
            name: "".to_string(),
            english_name: "".to_string(),
            position: [0.0; 3],
            parent: 0,
            deform_depth: 0,
            boneflag: BoneFlags::from_bits_truncate(0),
            offset: [0.0; 3],
            child: 0,
            append_bone_index: 0,
            append_weight: 0.0,
            fixed_axis: [0.0; 3],
            local_axis_x: [0.0; 3],
            local_axis_z: [0.0; 3],
            key_value: 0,
            ik_target_index: 0,
            ik_iter_count: 0,
            ik_limit: 0.0,
            ik_links: vec![],
        };
        ctx.name = self.inner.read_text_buf(encode);
        ctx.english_name = self.inner.read_text_buf(encode);
        ctx.position = self.inner.read_vec3();
        ctx.parent = self.inner.read_sized(s_bone_index);
        ctx.deform_depth = self.inner.read_i32();
        ctx.boneflag = BoneFlags::from_bits_truncate(self.inner.read_u16());

        if ctx.boneflag.intersects(BoneFlags::CONNECT_TO_OTHER_BONE) {
            ctx.child = self.inner.read_sized(s_bone_index);
        } else {
            ctx.offset = self.inner.read_vec3();
        }
        if ctx
            .boneflag
            .intersects(BoneFlags::INHERIT_TRANSLATION | BoneFlags::INHERIT_ROTATION)
        {
            ctx.append_bone_index = self.inner.read_sized(s_bone_index);
            ctx.append_weight = self.inner.read_f32();
        }
        if ctx.boneflag.intersects(BoneFlags::FIXED_AXIS) {
            ctx.fixed_axis = self.inner.read_vec3();
        }
        if ctx.boneflag.intersects(BoneFlags::LOCAL_COORDINATE) {
            ctx.local_axis_x = self.inner.read_vec3();
            ctx.local_axis_z = self.inner.read_vec3();
        }
        if ctx.boneflag.intersects(BoneFlags::EXTERNAL_PARENT_DEFORM) {
            ctx.key_value = self.inner.read_i32();
        }
        if ctx.boneflag.intersects(BoneFlags::IK) {
            ctx.ik_target_index = self.inner.read_sized(s_bone_index);
            ctx.ik_iter_count = self.inner.read_i32();
            ctx.ik_limit = self.inner.read_f32();
            ctx.ik_links = (0..self.inner.read_i32())
                .map(|_| self.read_iklink())
                .collect();
        }

        ctx
    }
    fn read_iklink(&mut self) -> IKLink {
        let mut ctx = IKLink {
            ik_bone_index: 0,
            enable_limit: 0,
            limit_min: [0.0; 3],
            limit_max: [0.0; 3],
        };
        ctx.ik_bone_index = self.inner.read_sized(self.header.s_bone_index);
        ctx.enable_limit = self.inner.read_u8();
        if ctx.enable_limit == 1 {
            ctx.limit_min = self.inner.read_vec3();
            ctx.limit_max = self.inner.read_vec3();
        }
        ctx
    }
}

pub struct MorphsStage {
    header: Header,
    inner: BinaryReader,
}
impl MorphsStage {
    pub fn read(mut self) -> (Vec<Morph>, FrameStage) {
        (
            (0..self.inner.read_i32())
                .map(|_| self.read_pmx_morph())
                .collect(),
            FrameStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }

    fn read_pmx_morph(&mut self) -> Morph {
        let mut ctx = Morph {
            name: "".to_string(),
            english_name: "".to_string(),
            category: 0,
            morph_type: 0,
            offset: 0,
            morph_data: vec![],
        };
        let encode = self.header.encode;
        ctx.name = self.inner.read_text_buf(encode);
        ctx.english_name = self.inner.read_text_buf(encode);
        ctx.category = self.inner.read_u8();
        ctx.morph_type = self.inner.read_u8();
        ctx.offset = self.inner.read_i32();
        let mut v = vec![];
        for _ in 0..ctx.offset {
            let morph = match ctx.morph_type {
                0 => MorphKinds::Group(self.read_group_morph()),
                1 => MorphKinds::Vertex(self.read_vertex_morph()),
                2 => MorphKinds::Bone(self.read_bone_morph()),
                3 => MorphKinds::UV(self.read_uv_morph()),
                4 => MorphKinds::UV1(self.read_uv_morph()),
                5 => MorphKinds::UV2(self.read_uv_morph()),
                6 => MorphKinds::UV3(self.read_uv_morph()),
                7 => MorphKinds::UV4(self.read_uv_morph()),
                8 => MorphKinds::Material(self.read_material_morph()),
                _ => panic!("Unexpected morph type:{}", ctx.morph_type),
            };
            v.push(morph);
        }
        ctx.morph_data = v;
        ctx
    }
    fn read_vertex_morph(&mut self) -> VertexMorph {
        let mut ctx = VertexMorph {
            index: 0,
            offset: [0.0; 3],
        };
        ctx.index = self.inner.read_vertex_index(self.header.s_vertex_index);
        ctx.offset = self.inner.read_vec3();
        ctx
    }
    fn read_uv_morph(&mut self) -> UVMorph {
        let mut ctx = UVMorph {
            index: 0,
            offset: [0.0; 4],
        };
        ctx.index = self.inner.read_vertex_index(self.header.s_vertex_index);
        ctx.offset = self.inner.read_vec4();
        ctx
    }
    fn read_bone_morph(&mut self) -> BoneMorph {
        let mut ctx = BoneMorph {
            index: 0,
            translates: [0.0; 3],
            rotates: [0.0; 4],
        };
        ctx.index = self.inner.read_sized(self.header.s_bone_index);
        ctx.translates = self.inner.read_vec3();
        ctx.rotates = self.inner.read_vec4();
        ctx
    }
    fn read_material_morph(&mut self) -> MaterialMorph {
        let mut ctx = MaterialMorph {
            index: 0,
            formula: 0,
            diffuse: [0.0; 4],
            specular: [0.0; 3],
            specular_factor: 0.0,
            ambient: [0.0; 3],
            edge_color: [0.0; 4],
            edge_size: 0.0,
            texture_factor: [0.0; 4],
            sphere_texture_factor: [0.0; 4],
            toon_texture_factor: [0.0; 4],
        };
        ctx.index = self.inner.read_sized(self.header.s_material_index);
        ctx.formula = self.inner.read_u8();
        ctx.diffuse = self.inner.read_vec4();
        ctx.specular = self.inner.read_vec3();
        ctx.specular_factor = self.inner.read_f32();
        ctx.ambient = self.inner.read_vec3();
        ctx.edge_color = self.inner.read_vec4();
        ctx.edge_size = self.inner.read_f32();
        ctx.texture_factor = self.inner.read_vec4();
        ctx.sphere_texture_factor = self.inner.read_vec4();
        ctx.toon_texture_factor = self.inner.read_vec4();
        ctx
    }
    fn read_group_morph(&mut self) -> GroupMorph {
        let mut ctx = GroupMorph {
            index: 0,
            morph_factor: 0.0,
        };
        let size = self.header.s_morph_index;
        ctx.index = self.inner.read_sized(size);
        ctx.morph_factor = self.inner.read_f32();
        ctx
    }
}
pub struct FrameStage {
    header: Header,
    inner: BinaryReader,
}

impl FrameStage {
    /// read `MMD` controller
    /// # Panics
    /// * if contains invalid target
    pub fn read(mut self) -> (Vec<Frame>, RigidStage) {
        (
            (0..self.inner.read_i32())
                .map(|_| Frame {
                    name: self.inner.read_text_buf(self.header.encode),
                    name_en: self.inner.read_text_buf(self.header.encode),
                    is_special: self.inner.read_u8(),
                    inners: (0..self.inner.read_i32())
                        .map(|_| {
                            let target = self.inner.read_u8();
                            FrameInner {
                                target,
                                index: match target {
                                    0 => self.inner.read_sized(self.header.s_bone_index),
                                    1 => self.inner.read_sized(self.header.s_morph_index),
                                    _ => {
                                        panic!("Invalid frame ")
                                    }
                                },
                            }
                        })
                        .collect(),
                })
                .collect(),
            RigidStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}
pub struct RigidStage {
    header: Header,
    inner: BinaryReader,
}
impl RigidStage {
    pub fn get_header(&self) -> Header {
        self.header.clone()
    }
    pub fn read(mut self) -> (Vec<Rigid>, JointStage) {
        let len = self.inner.read_i32();
        let mut bodies = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = self.inner.read_text_buf(self.header.encode);
            let name_en = self.inner.read_text_buf(self.header.encode);
            let bone_index = self.inner.read_sized(self.header.s_bone_index);
            let group = self.inner.read_u8();
            let un_collision_group_flag = self.inner.read_u16();
            let form = match self.inner.read_u8() {
                0 => RigidForm::Sphere,
                1 => RigidForm::Box,
                2 => RigidForm::Capsule,
                _ => {
                    unreachable!("Invalid  file detected at rigid loader")
                }
            };
            let size = self.inner.read_vec3();
            let position = self.inner.read_vec3();
            let rotation = self.inner.read_vec3();
            let mass = self.inner.read_f32();
            let move_resist = self.inner.read_f32();
            let rotation_resist = self.inner.read_f32();
            let repulsion = self.inner.read_f32();
            let friction = self.inner.read_f32();
            let calc_method = match self.inner.read_u8() {
                0 => RigidCalcMethod::Static,
                1 => RigidCalcMethod::Dynamic,
                2 => RigidCalcMethod::DynamicWithBonePosition,
                _ => {
                    unreachable!("Invalid  file detected as rigid loader")
                }
            };
            bodies.push(Rigid {
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
            });
        }
        (
            bodies,
            JointStage {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}

pub struct JointStage {
    header: Header,
    inner: BinaryReader,
}

impl JointStage {
    pub fn get_header(&self) -> Header {
        self.header.clone()
    }
    pub fn read(mut self) -> (Vec<Joint>, Option<SoftBodyStage>) {
        let len = self.inner.read_i32();
        let mut joints = Vec::with_capacity(len as usize);
        for _ in 0..len {
            joints.push(self.read_joint());
        }
        let next_loader = if self.header.version > 2.0 {
            //this file contains softbody section
            Some(SoftBodyStage {
                header: self.header,
                inner: self.inner,
            })
        } else {
            None
        };
        (joints, next_loader)
    }
    fn read_joint(&mut self) -> Joint {
        fn is_eq_f32(lhs: f32, rhs: f32) -> bool {
            (lhs - rhs).abs() < 0.01
        }
        let name = self.inner.read_text_buf(self.header.encode);
        let name_en = self.inner.read_text_buf(self.header.encode);
        let raw_parameter = {
            JointParameterRaw {
                joint_type: self.inner.read_u8(),
                a_rigid_index: self.inner.read_sized(self.header.s_rigid_body_index),
                b_rigid_index: self.inner.read_sized(self.header.s_rigid_body_index),
                position: self.inner.read_vec3(),
                rotation: self.inner.read_vec3(),
                move_limit_down: self.inner.read_vec3(),
                move_limit_up: self.inner.read_vec3(),
                rotation_limit_down: self.inner.read_vec3(),
                rotation_limit_up: self.inner.read_vec3(),
                spring_const_move: self.inner.read_vec3(),
                spring_const_rotation: self.inner.read_vec3(),
            }
        };

        let joint_parameter = match raw_parameter.joint_type {
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
        };
        Joint {
            name,
            name_en,
            joint_type: joint_parameter,
        }
    }
}

pub struct SoftBodyStage {
    header: Header,
    inner: BinaryReader,
}

impl SoftBodyStage {
    pub fn get_header(&self) -> Header {
        self.header.clone()
    }
    pub fn read(mut self) -> Vec<SoftBody> {
        (0..self.inner.read_i32())
            .map(|_| self.read_soft_body())
            .collect()
    }
    fn read_soft_body(&mut self) -> SoftBody {
        SoftBody {
            name: self.inner.read_text_buf(self.header.encode),
            name_en: self.inner.read_text_buf(self.header.encode),
            form: match self.inner.read_u8() {
                0 => SoftBodyForm::TriMesh,
                1 => SoftBodyForm::Rope,
                _ => {
                    panic!("Error invalid SoftBodyForm ")
                }
            },
            material_index: self.inner.read_sized(self.header.s_material_index),
            group: self.inner.read_u8(),
            un_collision_group_flag: self.inner.read_u16(),
            bit_flag: self.inner.read_u8(),
            b_link_create_distance: self.inner.read_i32(),
            clusters: self.inner.read_i32(),
            mass: self.inner.read_f32(),
            collision_margin: self.inner.read_f32(),
            aero_model: match self.inner.read_i32() {
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
            vcf: self.inner.read_f32(),
            dp: self.inner.read_f32(),
            dg: self.inner.read_f32(),
            lf: self.inner.read_f32(),
            pr: self.inner.read_f32(),
            vc: self.inner.read_f32(),
            df: self.inner.read_f32(),
            mt: self.inner.read_f32(),
            chr: self.inner.read_f32(),
            khr: self.inner.read_f32(),
            shr: self.inner.read_f32(),
            ahr: self.inner.read_f32(),
            //cluster
            srhr_cl: self.inner.read_f32(),
            skhr_cl: self.inner.read_f32(),
            sshr_cl: self.inner.read_f32(),
            sr_splt_cl: self.inner.read_f32(),
            sk_splt_cl: self.inner.read_f32(),
            ss_splt_cl: self.inner.read_f32(),
            //iteration
            v_it: self.inner.read_i32(),
            p_it: self.inner.read_i32(),
            d_it: self.inner.read_i32(),
            c_it: self.inner.read_i32(),
            //material
            lst: self.inner.read_f32(),
            ast: self.inner.read_f32(),
            vst: self.inner.read_f32(),
            anchor_rigid: (0..self.inner.read_i32())
                .map(|_| SoftBodyAnchorRigid {
                    rigid_index: self.inner.read_sized(self.header.s_rigid_body_index),
                    vertex_index: self.inner.read_vertex_index(self.header.s_vertex_index),
                    near_mode: match self.inner.read_i8() {
                        0 => true,
                        1 => false,
                        x => {
                            panic!("invalid near mode {}", x)
                        }
                    },
                })
                .collect(),
            pin_vertex: (0..self.inner.read_i32())
                .map(|_| self.inner.read_vertex_index(self.header.s_vertex_index))
                .collect(),
        }
    }
}
