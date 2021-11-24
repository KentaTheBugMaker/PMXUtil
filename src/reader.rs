//! # PMX loading module.
//! this module separated to some parts.To avoid invalid loading.
//!
//! |Current stage|product|Next stage|
//! |-------------|-------|----------|
//! |[ModelInfoReader]|[PMXModelInfo]|[VerticesReader]|
//! |[VerticesReader]|[`Vec<PMXVertex>`]|[FacesReader]|
//! |[FacesReader]|[`Vec<PMXFace>`]|[TexturesReader]|
//! |[TexturesReader]|[PMXTextureList]|[MaterialsReader]|
//! |[MaterialsReader]|[`Vec<PMXMaterial>`]|[BonesReader]|
//! |[BonesReader]|[`Vec<PMXBone>`]|[MorphsReader]|
//! |[MorphsReader]|[`Vec<PMXMorph>`]|[FrameReader]|
//! |[FrameReader]|[`Vec<PMXFrame>`]|[RigidReader]|
//! |[RigidReader]|[`Vec<PMXRigid>`]|[`JointReader`]|
//! |[JointReader]|[`Vec<PMXJoint>`]|[`Option<SoftBodyReader>`]|
//!
//! ```rust
//! // i want to get pmx path from env vars.
//! let path = std::env::var("PMX_FILE").unwrap();
//! let model_info_loader=pmx_util::reader::PMXReader::open(path);
//! let (model_info,vertices_loader)=model_info_loader.unwrap().read();
//! ```
//!

use std::path::Path;

use crate::binary_reader::BinaryReader;
use crate::types::{
    BoneMorph, Encode, FrameInner, GroupMorph, MaterialMorph, MorphTypes, PMXBone, PMXBoneFlags,
    PMXFace, PMXFrame, PMXHeader, PMXHeaderRaw, PMXIKLink, PMXJoint, PMXJointParameterRaw,
    PMXJointType, PMXMaterial, PMXMaterialFlags, PMXModelInfo, PMXMorph, PMXRigid,
    PMXRigidCalcMethod, PMXRigidForm, PMXSoftBody, PMXSoftBodyAeroModel, PMXSoftBodyAnchorRigid,
    PMXSoftBodyForm, PMXSphereMode, PMXSphereModeKind, PMXTextureList, PMXToonMode, PMXVertex,
    PMXVertexWeight, UVMorph, VertexMorph,
};

///
/// translate rustic header from raw header
///
/// # Arguments
///
/// * `header`: c header
///
/// returns: Option<PMXHeaderRust>
///
/// if invalid signature detected return None.
///
fn transform_header_c2r(header: PMXHeaderRaw) -> Option<PMXHeader> {
    let mut ctx = PMXHeader {
        magic: "".to_string(),
        version: 0.0,
        length: 0,
        encode: Encode::UTF8,
        additional_uv: 0,
        s_vertex_index: 0,
        s_texture_index: 0,
        s_material_index: 0,
        s_bone_index: 0,
        s_morph_index: 0,
        s_rigid_body_index: 0,
    };
    ctx.magic = String::from_utf8_lossy(&header.magic).to_string();
    // if invalid signature detected
    if ctx.magic != *"PMX " {
        return None;
    }
    ctx.version = header.version;
    ctx.length = header.length;
    ctx.encode = match header.config[0] {
        1 => Encode::UTF8,
        0 => Encode::Utf16Le,
        _ => panic!("Unknown Text Encoding"),
    };
    ctx.additional_uv = header.config[1];
    ctx.s_vertex_index = header.config[2];
    ctx.s_texture_index = header.config[3];
    ctx.s_material_index = header.config[4];
    ctx.s_bone_index = header.config[5];
    ctx.s_morph_index = header.config[6];
    ctx.s_rigid_body_index = header.config[7];
    Some(ctx)
}

pub struct PMXReader {
    header: PMXHeader,
}
impl PMXReader {
    /// the start of reader module.
    /// # None
    /// * invalid path given
    /// * read PMX magic number is not `PMX `
    /// # Arguments
    ///
    /// * `path`: path to pmx file.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// let path = std::env::var("PMX_FILE").unwrap();
    /// let model_info_loader = pmx_util::reader::PMXReader::open(path).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Option<ModelInfoReader> {
        let mut inner = BinaryReader::open(path).ok()?;
        let header = inner.read_raw_header();
        let header_rs = transform_header_c2r(header)?;
        Some(ModelInfoReader {
            header: header_rs,
            inner,
        })
    }
    /// Get Header Information
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }
}
pub struct ModelInfoReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl ModelInfoReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }

    pub fn read(mut self) -> (PMXModelInfo, VerticesReader) {
        let enc = self.header.encode;
        (
            PMXModelInfo {
                name: self.inner.read_text_buf(enc),
                name_en: self.inner.read_text_buf(enc),
                comment: self.inner.read_text_buf(enc),
                comment_en: self.inner.read_text_buf(enc),
            },
            VerticesReader {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}
pub struct VerticesReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl VerticesReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }

    pub fn read(mut self) -> (Vec<PMXVertex>, FacesReader) {
        let verts = self.inner.read_i32();
        let mut v = Vec::with_capacity(verts as usize);
        for _ in 0..verts {
            v.push(self.read_pmx_vertex());
        }
        assert_eq!(verts as usize, v.len());
        let faceloader = FacesReader {
            header: self.header,
            inner: self.inner,
        };
        (v, faceloader)
    }

    fn read_pmx_vertex(&mut self) -> PMXVertex {
        let mut ctx = PMXVertex {
            position: [0.0f32; 3],
            norm: [0.0f32; 3],
            uv: [0.0f32; 2],
            add_uv: [[0.0f32; 4]; 4],
            weight_type: PMXVertexWeight::BDEF1(-1),
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
                let index = self.inner.read_sized(size).unwrap();
                PMXVertexWeight::BDEF1(index)
            }

            1 => {
                let bone_index_1 = self.inner.read_sized(size).unwrap();
                let bone_index_2 = self.inner.read_sized(size).unwrap();
                let bone_weight_1 = self.inner.read_f32();
                PMXVertexWeight::BDEF2 {
                    bone_index_1,
                    bone_index_2,
                    bone_weight_1,
                }
            }
            2 => {
                let bone_index_1 = self.inner.read_sized(size).unwrap();
                let bone_index_2 = self.inner.read_sized(size).unwrap();
                let bone_index_3 = self.inner.read_sized(size).unwrap();
                let bone_index_4 = self.inner.read_sized(size).unwrap();
                let bone_weight_1 = self.inner.read_f32();
                let bone_weight_2 = self.inner.read_f32();
                let bone_weight_3 = self.inner.read_f32();
                let bone_weight_4 = self.inner.read_f32();
                PMXVertexWeight::BDEF4 {
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

            3 => {
                let bone_index_1 = self.inner.read_sized(size).unwrap();
                let bone_index_2 = self.inner.read_sized(size).unwrap();
                let bone_weight_1 = self.inner.read_f32();
                let sdef_c = self.inner.read_vec3();
                let sdef_r0 = self.inner.read_vec3();
                let sdef_r1 = self.inner.read_vec3();
                PMXVertexWeight::SDEF {
                    bone_index_1,
                    bone_index_2,
                    bone_weight_1,
                    sdef_c,
                    sdef_r0,
                    sdef_r1,
                }
            }
            4 => {
                let bone_index_1 = self.inner.read_sized(size).unwrap();
                let bone_index_2 = self.inner.read_sized(size).unwrap();
                let bone_index_3 = self.inner.read_sized(size).unwrap();
                let bone_index_4 = self.inner.read_sized(size).unwrap();
                let bone_weight_1 = self.inner.read_f32();
                let bone_weight_2 = self.inner.read_f32();
                let bone_weight_3 = self.inner.read_f32();
                let bone_weight_4 = self.inner.read_f32();
                PMXVertexWeight::QDEF {
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
            _ => {
                panic!("Unknown Weight type:{}", weight_type);
            }
        };

        ctx.edge_mag = self.inner.read_f32();
        ctx
    }
}
pub struct FacesReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl FacesReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }

    pub fn read(mut self) -> (Vec<PMXFace>, TexturesReader) {
        let mut v = vec![];
        let faces = self.inner.read_i32();
        let s_vertex_index = self.header.s_vertex_index;
        let faces = faces / 3;
        for _ in 0..faces {
            let v0 = self.inner.read_vertex_index(s_vertex_index).unwrap();
            let v1 = self.inner.read_vertex_index(s_vertex_index).unwrap();
            let v2 = self.inner.read_vertex_index(s_vertex_index).unwrap();
            v.push(PMXFace {
                vertices: [v0, v1, v2],
            });
        }
        assert_eq!(v.len(), faces as usize);
        let next_self = TexturesReader {
            header: self.header,
            inner: self.inner,
        };
        (v, next_self)
    }
}

pub struct TexturesReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl TexturesReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }

    /// Read relative texture path from current reading file
    pub fn read(mut self) -> (PMXTextureList, MaterialsReader) {
        let textures = self.inner.read_i32();
        let mut v = vec![];
        for _ in 0..textures {
            v.push(self.inner.read_text_buf(self.header.encode));
        }
        let next_self = MaterialsReader {
            header: self.header,
            inner: self.inner,
        };
        (PMXTextureList { textures: v }, next_self)
    }
}
pub struct MaterialsReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl MaterialsReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }
    ///Read material's information contains name ambient diffuse specular etc parameters.
    /// for exact model rendering you should passed to shader and processed proper.
    pub fn read(mut self) -> (Vec<PMXMaterial>, BonesReader) {
        let mut materials = vec![];
        let counts = self.inner.read_i32();
        for _ in 0..counts {
            let material = self.read_pmx_material();
            materials.push(material);
        }
        let next_self = BonesReader {
            header: self.header,
            inner: self.inner,
        };
        (materials, next_self)
    }

    fn read_pmx_material(&mut self) -> PMXMaterial {
        let s_texture_index = self.header.s_texture_index;
        let mut ctx = PMXMaterial {
            name: "".to_string(),
            english_name: "".to_string(),
            diffuse: [0.0f32; 4],
            specular: [0.0f32; 3],
            specular_factor: 0.0,
            ambient: [0.0f32; 3],
            draw_mode: PMXMaterialFlags::from_bits_truncate(0),
            edge_color: [0.0f32; 4],
            edge_size: 0.0,
            texture_index: 0,
            sphere_mode: None,
            toon_mode: PMXToonMode::Common(0),
            memo: "".to_string(),
            num_face_vertices: 0,
        };
        ctx.name = self.inner.read_text_buf(self.header.encode);
        ctx.english_name = self.inner.read_text_buf(self.header.encode);
        ctx.diffuse = self.inner.read_vec4();
        ctx.specular = self.inner.read_vec3();
        ctx.specular_factor = self.inner.read_f32();
        ctx.ambient = self.inner.read_vec3();
        ctx.draw_mode = PMXMaterialFlags::from_bits_truncate(self.inner.read_u8());
        ctx.edge_color = self.inner.read_vec4();
        ctx.edge_size = self.inner.read_f32();
        ctx.texture_index = self.inner.read_sized(s_texture_index).unwrap();
        let ti = self.inner.read_sized(s_texture_index).unwrap();
        let spmode = self.inner.read_u8();
        ctx.sphere_mode = match spmode {
            0 => None,
            1 => Some(PMXSphereMode {
                index: ti,
                kind: PMXSphereModeKind::Mul,
            }),
            2 => Some(PMXSphereMode {
                index: ti,
                kind: PMXSphereModeKind::Add,
            }),
            3 => Some(PMXSphereMode {
                index: ti,
                kind: PMXSphereModeKind::SubTexture,
            }),
            _ => {
                panic!("Error Unknown SphereMode:{}", spmode);
            }
        };

        let toonmode = self.inner.read_u8();
        ctx.toon_mode = match toonmode {
            0 => PMXToonMode::Separate(self.inner.read_sized(s_texture_index).unwrap()),
            1 => PMXToonMode::Common(self.inner.read_u8()),
            _ => panic!("Error Unknown Toon flag:{}", toonmode),
        };
        ctx.memo = self.inner.read_text_buf(self.header.encode);
        ctx.num_face_vertices = self.inner.read_i32();
        ctx
    }
}
pub struct BonesReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl BonesReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }

    /// read bone's information parent child IK etc.
    /// Exact model pose you should process this parameter and pass to PhisicsEngine e.g. bullet havok
    pub fn read(mut self) -> (Vec<PMXBone>, MorphsReader) {
        let mut bones = vec![];
        let count = self.inner.read_i32();

        for _ in 0..count {
            let bone = self.read_pmx_bone();
            bones.push(bone);
        }

        let next_self = MorphsReader {
            header: self.header,
            inner: self.inner,
        };
        (bones, next_self)
    }
    fn read_pmx_bone(&mut self) -> PMXBone {
        let encode = self.header.encode;
        let s_bone_index = self.header.s_bone_index;
        let mut ctx = PMXBone {
            name: "".to_string(),
            english_name: "".to_string(),
            position: [0.0f32; 3],
            parent: 0,
            deform_depth: 0,
            boneflag: PMXBoneFlags::from_bits_truncate(0),
            offset: [0.0f32; 3],
            child: 0,
            append_bone_index: 0,
            append_weight: 0.0,
            fixed_axis: [0.0f32; 3],
            local_axis_x: [0.0f32; 3],
            local_axis_z: [0.0f32; 3],
            key_value: 0,
            ik_target_index: 0,
            ik_iter_count: 0,
            ik_limit: 0.0,
            ik_links: vec![],
        };
        ctx.name = self.inner.read_text_buf(encode);
        ctx.english_name = self.inner.read_text_buf(encode);
        ctx.position = self.inner.read_vec3();
        ctx.parent = self.inner.read_sized(s_bone_index).unwrap();
        ctx.deform_depth = self.inner.read_i32();
        ctx.boneflag = PMXBoneFlags::from_bits_truncate(self.inner.read_u16());

        if ctx.boneflag.intersects(PMXBoneFlags::CONNECT_TO_OTHER_BONE) {
            ctx.child = self.inner.read_sized(s_bone_index).unwrap();
        } else {
            ctx.offset = self.inner.read_vec3();
        }
        if ctx
            .boneflag
            .intersects(PMXBoneFlags::INHERIT_TRANSLATION | PMXBoneFlags::INHERIT_ROTATION)
        {
            ctx.append_bone_index = self.inner.read_sized(s_bone_index).unwrap();
            ctx.append_weight = self.inner.read_f32();
        }
        if ctx.boneflag.intersects(PMXBoneFlags::FIXED_AXIS) {
            ctx.fixed_axis = self.inner.read_vec3();
        }
        if ctx.boneflag.intersects(PMXBoneFlags::LOCAL_COORDINATE) {
            ctx.local_axis_x = self.inner.read_vec3();
            ctx.local_axis_z = self.inner.read_vec3();
        }
        if ctx
            .boneflag
            .intersects(PMXBoneFlags::EXTERNAL_PARENT_DEFORM)
        {
            ctx.key_value = self.inner.read_i32();
        }
        if ctx.boneflag.intersects(PMXBoneFlags::IK) {
            ctx.ik_target_index = self.inner.read_sized(s_bone_index).unwrap();
            ctx.ik_iter_count = self.inner.read_i32();
            ctx.ik_limit = self.inner.read_f32();
            for _ in 0..self.inner.read_i32() {
                ctx.ik_links.push(self.read_iklink())
            }
        }

        ctx
    }
    fn read_iklink(&mut self) -> PMXIKLink {
        let mut ctx = PMXIKLink {
            ik_bone_index: 0,
            enable_limit: 0,
            limit_min: [0.0f32; 3],
            limit_max: [0.0f32; 3],
        };
        ctx.ik_bone_index = self.inner.read_sized(self.header.s_bone_index).unwrap();
        ctx.enable_limit = self.inner.read_u8();
        if ctx.enable_limit == 1 {
            ctx.limit_min = self.inner.read_vec3();
            ctx.limit_max = self.inner.read_vec3();
        }
        ctx
    }
}

pub struct MorphsReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl MorphsReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }
    pub fn read(mut self) -> (Vec<PMXMorph>, FrameReader) {
        (
            (0..self.inner.read_i32())
                .map(|_| self.read_pmx_morph())
                .collect(),
            FrameReader {
                header: self.header,
                inner: self.inner,
            },
        )
    }

    fn read_pmx_morph(&mut self) -> PMXMorph {
        let mut ctx = PMXMorph {
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
                0 => MorphTypes::Group(self.read_group_morph()),
                1 => MorphTypes::Vertex(self.read_vertex_morph()),
                2 => MorphTypes::Bone(self.read_bone_morph()),
                3 => MorphTypes::UV(self.read_uv_morph()),
                4 => MorphTypes::UV1(self.read_uv_morph()),
                5 => MorphTypes::UV2(self.read_uv_morph()),
                6 => MorphTypes::UV3(self.read_uv_morph()),
                7 => MorphTypes::UV4(self.read_uv_morph()),
                8 => MorphTypes::Material(self.read_material_morph()),
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
            offset: [0.0f32; 3],
        };
        ctx.index = self.inner.read_sized(self.header.s_vertex_index).unwrap();
        ctx.offset = self.inner.read_vec3();
        ctx
    }
    fn read_uv_morph(&mut self) -> UVMorph {
        let mut ctx = UVMorph {
            index: 0,
            offset: [0.0f32; 4],
        };
        ctx.index = self.inner.read_sized(self.header.s_vertex_index).unwrap();
        ctx.offset = self.inner.read_vec4();
        ctx
    }
    fn read_bone_morph(&mut self) -> BoneMorph {
        let mut ctx = BoneMorph {
            index: 0,
            translates: [0.0f32; 3],
            rotates: [0.0f32; 4],
        };
        ctx.index = self.inner.read_sized(self.header.s_bone_index).unwrap();
        ctx.translates = self.inner.read_vec3();
        ctx.rotates = self.inner.read_vec4();
        ctx
    }
    fn read_material_morph(&mut self) -> MaterialMorph {
        let mut ctx = MaterialMorph {
            index: 0,
            formula: 0,
            diffuse: [0.0f32; 4],
            specular: [0.0f32; 3],
            specular_factor: 0.0,
            ambient: [0.0f32; 3],
            edge_color: [0.0f32; 4],
            edge_size: 0.0,
            texture_factor: [0.0f32; 4],
            sphere_texture_factor: [0.0f32; 4],
            toon_texture_factor: [0.0f32; 4],
        };
        ctx.index = self.inner.read_sized(self.header.s_material_index).unwrap();
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
        ctx.index = self.inner.read_sized(size).unwrap();
        ctx.morph_factor = self.inner.read_f32();
        ctx
    }
}
pub struct FrameReader {
    header: PMXHeader,
    inner: BinaryReader,
}

impl FrameReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }
    /// read MMD 's controller
    /// next stage is RigidReader
    pub fn read(mut self) -> (Vec<PMXFrame>, RigidReader) {
        let count = self.inner.read_i32();
        let mut frames = vec![];
        for _ in 0..count {
            let name = self.inner.read_text_buf(self.header.encode);
            let name_en = self.inner.read_text_buf(self.header.encode);
            let flag = self.inner.read_u8();
            let count = self.inner.read_i32();
            let mut o = vec![];
            for _ in 0..count {
                let target = self.inner.read_u8();
                let index = match target {
                    0 => self.inner.read_sized(self.header.s_bone_index).unwrap(),
                    1 => self.inner.read_sized(self.header.s_morph_index).unwrap(),
                    _ => {
                        panic!("Invalid frame ")
                    }
                };
                o.push(FrameInner { target, index })
            }
            frames.push(PMXFrame {
                name,
                name_en,
                is_special: flag,
                inners: o,
            })
        }
        (
            frames,
            RigidReader {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}
pub struct RigidReader {
    header: PMXHeader,
    inner: BinaryReader,
}
impl RigidReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }
    pub fn read(mut self) -> (Vec<PMXRigid>, JointReader) {
        let len = self.inner.read_i32();
        let mut bodies = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = self.inner.read_text_buf(self.header.encode);
            let name_en = self.inner.read_text_buf(self.header.encode);
            let bone_index = self.inner.read_sized(self.header.s_bone_index).unwrap();
            let group = self.inner.read_u8();
            let un_collision_group_flag = self.inner.read_u16();
            let form = match self.inner.read_u8() {
                0 => PMXRigidForm::Sphere,
                1 => PMXRigidForm::Box,
                2 => PMXRigidForm::Capsule,
                _ => {
                    unreachable!("Invalid PMX file detected at rigid loader")
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
                0 => PMXRigidCalcMethod::Static,
                1 => PMXRigidCalcMethod::Dynamic,
                2 => PMXRigidCalcMethod::DynamicWithBonePosition,
                _ => {
                    unreachable!("Invalid PMX file detected as rigid loader")
                }
            };
            bodies.push(PMXRigid {
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
            JointReader {
                header: self.header,
                inner: self.inner,
            },
        )
    }
}

pub struct JointReader {
    header: PMXHeader,
    inner: BinaryReader,
}

impl JointReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }
    pub fn read(mut self) -> (Vec<PMXJoint>, Option<SoftBodyReader>) {
        let len = self.inner.read_i32();
        let mut joints = Vec::with_capacity(len as usize);
        for _ in 0..len {
            joints.push(self.read_joint());
        }
        let next_loader = if self.header.version > 2.0 {
            //this file contains softbody section
            Some(SoftBodyReader {
                header: self.header,
                inner: self.inner,
            })
        } else {
            None
        };
        (joints, next_loader)
    }
    fn read_joint(&mut self) -> PMXJoint {
        let name = self.inner.read_text_buf(self.header.encode);
        let name_en = self.inner.read_text_buf(self.header.encode);
        let raw_parameter = {
            PMXJointParameterRaw {
                joint_type: self.inner.read_u8(),
                a_rigid_index: self
                    .inner
                    .read_sized(self.header.s_rigid_body_index)
                    .unwrap(),
                b_rigid_index: self
                    .inner
                    .read_sized(self.header.s_rigid_body_index)
                    .unwrap(),
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
        fn is_eq_f32(lhs: f32, rhs: f32) -> bool {
            (lhs - rhs).abs() < 0.01
        }
        let joint_parameter = match raw_parameter.joint_type {
            0 => PMXJointType::Spring6DOF {
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
            1 => PMXJointType::SixDof {
                a_rigid_index: raw_parameter.a_rigid_index,
                b_rigid_index: raw_parameter.b_rigid_index,
                position: raw_parameter.position,
                rotation: raw_parameter.rotation,
                move_limit_down: raw_parameter.move_limit_down,
                move_limit_up: raw_parameter.move_limit_up,
                rotation_limit_down: raw_parameter.rotation_limit_down,
                rotation_limit_up: raw_parameter.rotation_limit_up,
            },
            2 => PMXJointType::P2P {
                a_rigid_index: raw_parameter.a_rigid_index,
                b_rigid_index: raw_parameter.b_rigid_index,
                position: raw_parameter.position,
                rotation: raw_parameter.rotation,
            },
            3 => PMXJointType::ConeTwist {
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
            4 => PMXJointType::Slider {
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
            5 => PMXJointType::Hinge {
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
        PMXJoint {
            name,
            name_en,
            joint_type: joint_parameter,
        }
    }
}

pub struct SoftBodyReader {
    header: PMXHeader,
    inner: BinaryReader,
}

impl SoftBodyReader {
    pub fn get_header(&self) -> PMXHeader {
        self.header.clone()
    }
    pub fn read(mut self) -> Vec<PMXSoftBody> {
        let n_soft_bodies = self.inner.read_i32();
        let mut soft_bodies = Vec::with_capacity(n_soft_bodies as usize);
        for _ in 0..n_soft_bodies {
            soft_bodies.push(self.read_soft_body())
        }
        soft_bodies
    }
    fn read_soft_body(&mut self) -> PMXSoftBody {
        let name = self.inner.read_text_buf(self.header.encode);
        let name_en = self.inner.read_text_buf(self.header.encode);
        let form = match self.inner.read_u8() {
            0 => PMXSoftBodyForm::TriMesh,
            1 => PMXSoftBodyForm::Rope,
            _ => {
                panic!("Error invalid PMXSoftBodyForm ")
            }
        };
        let material_index = self.inner.read_sized(self.header.s_material_index).unwrap();
        let group = self.inner.read_u8();
        let un_collision_group_flag = self.inner.read_u16();
        let bit_flag = self.inner.read_u8();
        let b_link_create_distance = self.inner.read_i32();
        let clusters = self.inner.read_i32();
        let mass = self.inner.read_f32();
        let collision_margin = self.inner.read_f32();
        let aero_model = match self.inner.read_i32() {
            0 => PMXSoftBodyAeroModel::VPoint,
            1 => PMXSoftBodyAeroModel::VTwoSide,
            2 => PMXSoftBodyAeroModel::VOneSided,
            3 => PMXSoftBodyAeroModel::FTwoSided,
            4 => PMXSoftBodyAeroModel::FOneSided,
            _ => {
                panic!("Error invalid PMXSoftBodyAeroModel")
            }
        };
        //config
        let vcf = self.inner.read_f32();
        let dp = self.inner.read_f32();
        let dg = self.inner.read_f32();
        let lf = self.inner.read_f32();
        let pr = self.inner.read_f32();
        let vc = self.inner.read_f32();
        let df = self.inner.read_f32();
        let mt = self.inner.read_f32();
        let chr = self.inner.read_f32();
        let khr = self.inner.read_f32();
        let shr = self.inner.read_f32();
        let ahr = self.inner.read_f32();
        //cluster
        let srhr_cl = self.inner.read_f32();
        let skhr_cl = self.inner.read_f32();
        let sshr_cl = self.inner.read_f32();
        let sr_splt_cl = self.inner.read_f32();
        let sk_splt_cl = self.inner.read_f32();
        let ss_splt_cl = self.inner.read_f32();
        //iteration
        let v_it = self.inner.read_i32();
        let p_it = self.inner.read_i32();
        let d_it = self.inner.read_i32();
        let c_it = self.inner.read_i32();
        //material
        let lst = self.inner.read_f32();
        let ast = self.inner.read_f32();
        let vst = self.inner.read_f32();
        let n_anchor_rigid = self.inner.read_i32();
        let mut anchor_rigid = Vec::with_capacity(n_anchor_rigid as usize);
        for _ in 0..n_anchor_rigid {
            let rigid_index = self
                .inner
                .read_sized(self.header.s_rigid_body_index)
                .unwrap();
            let vertex_index = self
                .inner
                .read_vertex_index(self.header.s_vertex_index)
                .unwrap();
            let near_mode = match self.inner.read_i8() {
                0 => true,
                1 => false,
                _ => panic!("Error detected PMXSoftBodyAnchorRigid near mode"),
            };
            anchor_rigid.push(PMXSoftBodyAnchorRigid {
                rigid_index,
                vertex_index,
                near_mode,
            });
        }
        let n_pin_vertex = self.inner.read_i32();
        let mut pin_vertex = Vec::with_capacity(n_pin_vertex as usize);
        for _ in 0..n_pin_vertex {
            pin_vertex.push(
                self.inner
                    .read_vertex_index(self.header.s_vertex_index)
                    .unwrap(),
            )
        }
        PMXSoftBody {
            name,
            name_en,
            form,
            material_index,
            group,
            un_collision_group_flag,
            bit_flag,
            b_link_create_distance,
            clusters,
            mass,
            collision_margin,
            aero_model,
            vcf,
            dp,
            dg,
            lf,
            pr,
            vc,
            df,
            mt,
            chr,
            khr,
            shr,
            ahr,
            srhr_cl,
            skhr_cl,
            sshr_cl,
            sr_splt_cl,
            sk_splt_cl,
            ss_splt_cl,
            v_it,
            p_it,
            d_it,
            c_it,
            lst,
            ast,
            vst,
            anchor_rigid,
            pin_vertex,
        }
    }
}
