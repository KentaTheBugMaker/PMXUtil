use crate::binary_reader::BinaryReader;
use crate::pmx_types::pmx_types::{
    BoneMorph, GroupMorph, MaterialMorph, MorphTypes, PMXBone, PMXBones, PMXHeaderRust, PMXIKLink,
    PMXMaterial, PMXMaterials, PMXMorph, PMXMorphs, PMXSphereMode, PMXTextureList, PMXToonMode,
    UVMorph, VertexMorph, BONE_FLAG_APPEND_ROTATE_MASK, BONE_FLAG_APPEND_TRANSLATE_MASK,
    BONE_FLAG_DEFORM_OUTER_PARENT_MASK, BONE_FLAG_FIXED_AXIS_MASK, BONE_FLAG_IK_MASK,
    BONE_FLAG_LOCAL_AXIS_MASK, BONE_FLAG_TARGET_SHOW_MODE_MASK,
};

pub mod pmx_loader {
    use crate::binary_reader::BinaryReader;
    use crate::pmx_loader::TexturesLoader;
    use crate::pmx_types::pmx_types::{
        Encode, PMXFace, PMXFaces, PMXHeaderC, PMXHeaderRust, PMXModelInfo, PMXVertex,
        PMXVertexWeight, PMXVertices,
    };
    use std::path::Path;

    fn transform_header_c2r(header: PMXHeaderC) -> PMXHeaderRust {
        let mut ctx = PMXHeaderRust {
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
        ctx
    }

    pub struct PMXLoader {
        header: PMXHeaderRust,
        inner: BinaryReader,
    }
    impl PMXLoader {
        pub fn open<P: AsRef<Path>>(path: P) -> ModelInfoLoader {
            let mut inner = BinaryReader::open(path).unwrap();
            let header = inner.read_PMXHeader_raw();
            let header_rs = transform_header_c2r(header);
            ModelInfoLoader {
                header: header_rs,
                inner,
            }
        }
        pub fn get_header(&self) -> PMXHeaderRust {
            PMXHeaderRust {
                magic: "PMX ".to_string(),
                version: self.header.version,
                length: self.header.length,
                encode: self.header.encode,
                additional_uv: self.header.additional_uv,
                s_vertex_index: self.header.s_vertex_index,
                s_texture_index: self.header.s_texture_index,
                s_material_index: self.header.s_material_index,
                s_bone_index: self.header.s_bone_index,
                s_morph_index: self.header.s_morph_index,
                s_rigid_body_index: self.header.s_rigid_body_index,
            }
        }
    }
    pub struct ModelInfoLoader {
        header: PMXHeaderRust,
        inner: BinaryReader,
    }
    impl ModelInfoLoader {
        pub fn get_header(&self) -> PMXHeaderRust {
            PMXHeaderRust {
                magic: "PMX ".to_string(),
                version: self.header.version,
                length: self.header.length,
                encode: self.header.encode,
                additional_uv: self.header.additional_uv,
                s_vertex_index: self.header.s_vertex_index,
                s_texture_index: self.header.s_texture_index,
                s_material_index: self.header.s_material_index,
                s_bone_index: self.header.s_bone_index,
                s_morph_index: self.header.s_morph_index,
                s_rigid_body_index: self.header.s_rigid_body_index,
            }
        }
        pub fn read_pmx_model_info(mut stage: Self) -> (PMXModelInfo, VerticesLoader) {
            let mut ctx = PMXModelInfo {
                name: "".to_string(),
                name_en: "".to_string(),
                comment: "".to_string(),
                comment_en: "".to_string(),
            };
            let enc = stage.header.encode;
            ctx.name = stage.inner.read_text_buf(enc);
            ctx.name_en = stage.inner.read_text_buf(enc);
            ctx.comment = stage.inner.read_text_buf(enc);
            ctx.comment_en = stage.inner.read_text_buf(enc);
            let verticesloader = VerticesLoader {
                header: stage.header,
                inner: stage.inner,
            };
            (ctx, verticesloader)
        }
    }
    pub struct VerticesLoader {
        header: PMXHeaderRust,
        inner: BinaryReader,
    }
    impl VerticesLoader {
        pub fn get_header(&self) -> PMXHeaderRust {
            PMXHeaderRust {
                magic: "PMX ".to_string(),
                version: self.header.version,
                length: self.header.length,
                encode: self.header.encode,
                additional_uv: self.header.additional_uv,
                s_vertex_index: self.header.s_vertex_index,
                s_texture_index: self.header.s_texture_index,
                s_material_index: self.header.s_material_index,
                s_bone_index: self.header.s_bone_index,
                s_morph_index: self.header.s_morph_index,
                s_rigid_body_index: self.header.s_rigid_body_index,
            }
        }
        pub fn read_pmx_vertices(mut stage: Self) -> (PMXVertices, FacesLoader) {
            let mut ctx = PMXVertices { vertices: vec![] };
            let verts = stage.inner.read_i32();
            let mut v = Vec::with_capacity(verts as usize);
            for _ in 0..verts {
                v.push(stage.read_pmx_vertex());
            }
            assert_eq!(verts as usize, v.len());
            ctx.vertices = v;
            let faceloader = FacesLoader {
                header: stage.header,
                inner: stage.inner,
            };
            (ctx, faceloader)
        }

        fn read_pmx_vertex(&mut self) -> PMXVertex {
            let mut ctx = PMXVertex {
                position: [0.0f32; 3],
                norm: [0.0f32; 3],
                uv: [0.0f32; 2],
                add_uv: [[0.0f32; 4]; 4],
                weight_type: PMXVertexWeight::BDEF1,
                bone_indices: [-1i32; 4],
                bone_weights: [0.0f32; 4],
                sdef_c: [0.0f32; 3],
                sdef_r0: [0.0f32; 3],
                sdef_r1: [0.0f32; 3],
                edge_mag: 1.0,
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
                0 => PMXVertexWeight::BDEF1,
                1 => PMXVertexWeight::BDEF2,
                2 => PMXVertexWeight::BDEF4,
                3 => PMXVertexWeight::SDEF,
                4 => PMXVertexWeight::QDEF,
                _ => {
                    panic!("Unknown Weight type:{}", weight_type);
                }
            };
            match ctx.weight_type {
                PMXVertexWeight::BDEF1 => {
                    ctx.bone_indices[0] = self.inner.read_sized(size).unwrap();
                }
                PMXVertexWeight::BDEF2 => {
                    ctx.bone_indices[0] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[1] = self.inner.read_sized(size).unwrap();
                    ctx.bone_weights[0] = self.inner.read_f32();
                }
                PMXVertexWeight::BDEF4 => {
                    ctx.bone_indices[0] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[1] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[2] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[3] = self.inner.read_sized(size).unwrap();
                    ctx.bone_weights[0] = self.inner.read_f32();
                    ctx.bone_weights[1] = self.inner.read_f32();
                    ctx.bone_weights[2] = self.inner.read_f32();
                    ctx.bone_weights[3] = self.inner.read_f32();
                }
                PMXVertexWeight::SDEF => {
                    ctx.bone_indices[0] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[1] = self.inner.read_sized(size).unwrap();
                    ctx.bone_weights[0] = self.inner.read_f32();
                    ctx.sdef_c = self.inner.read_vec3();
                    ctx.sdef_r0 = self.inner.read_vec3();
                    ctx.sdef_r1 = self.inner.read_vec3();
                }
                PMXVertexWeight::QDEF => {
                    ctx.bone_indices[0] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[1] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[2] = self.inner.read_sized(size).unwrap();
                    ctx.bone_indices[3] = self.inner.read_sized(size).unwrap();
                    ctx.bone_weights[0] = self.inner.read_f32();
                    ctx.bone_weights[1] = self.inner.read_f32();
                    ctx.bone_weights[2] = self.inner.read_f32();
                    ctx.bone_weights[3] = self.inner.read_f32();
                }
            }
            ctx.edge_mag = self.inner.read_f32();
            ctx
        }
    }
    pub struct FacesLoader {
        header: PMXHeaderRust,
        inner: BinaryReader,
    }
    impl FacesLoader {
        pub fn get_header(&self) -> PMXHeaderRust {
            PMXHeaderRust {
                magic: "PMX ".to_string(),
                version: self.header.version,
                length: self.header.length,
                encode: self.header.encode,
                additional_uv: self.header.additional_uv,
                s_vertex_index: self.header.s_vertex_index,
                s_texture_index: self.header.s_texture_index,
                s_material_index: self.header.s_material_index,
                s_bone_index: self.header.s_bone_index,
                s_morph_index: self.header.s_morph_index,
                s_rigid_body_index: self.header.s_rigid_body_index,
            }
        }
        pub fn read_pmx_faces(mut stage: Self) -> (PMXFaces, TexturesLoader) {
            let mut ctx = PMXFaces { faces: vec![] };
            let faces = stage.inner.read_i32();
            let s_vertex_index = stage.header.s_vertex_index;
            let faces = faces / 3;
            for _ in 0..faces {
                let v0 = stage.inner.read_vertex_index(s_vertex_index).unwrap();
                let v1 = stage.inner.read_vertex_index(s_vertex_index).unwrap();
                let v2 = stage.inner.read_vertex_index(s_vertex_index).unwrap();
                ctx.faces.push(PMXFace {
                    vertices: [v0, v1, v2],
                });
            }
            assert_eq!(ctx.faces.len(), faces as usize);
            let next_stage = TexturesLoader {
                header: stage.header,
                inner: stage.inner,
            };
            (ctx, next_stage)
        }
    }
}
pub struct TexturesLoader {
    header: PMXHeaderRust,
    inner: BinaryReader,
}
impl TexturesLoader {
    pub fn get_header(&self) -> PMXHeaderRust {
        PMXHeaderRust {
            magic: "PMX ".to_string(),
            version: self.header.version,
            length: self.header.length,
            encode: self.header.encode,
            additional_uv: self.header.additional_uv,
            s_vertex_index: self.header.s_vertex_index,
            s_texture_index: self.header.s_texture_index,
            s_material_index: self.header.s_material_index,
            s_bone_index: self.header.s_bone_index,
            s_morph_index: self.header.s_morph_index,
            s_rigid_body_index: self.header.s_rigid_body_index,
        }
    }
    pub fn read_texture_list(mut stage: Self) -> (PMXTextureList, MaterialsLoader) {
        let textures = stage.inner.read_i32();
        let mut v = vec![];
        for _ in 0..textures {
            v.push(stage.inner.read_text_buf(stage.header.encode));
        }
        let next_stage = MaterialsLoader {
            header: stage.header,
            inner: stage.inner,
        };
        (PMXTextureList { textures: v }, next_stage)
    }
}
pub struct MaterialsLoader {
    header: PMXHeaderRust,
    inner: BinaryReader,
}
impl MaterialsLoader {
    pub fn get_header(&self) -> PMXHeaderRust {
        PMXHeaderRust {
            magic: "PMX ".to_string(),
            version: self.header.version,
            length: self.header.length,
            encode: self.header.encode,
            additional_uv: self.header.additional_uv,
            s_vertex_index: self.header.s_vertex_index,
            s_texture_index: self.header.s_texture_index,
            s_material_index: self.header.s_material_index,
            s_bone_index: self.header.s_bone_index,
            s_morph_index: self.header.s_morph_index,
            s_rigid_body_index: self.header.s_rigid_body_index,
        }
    }
    pub fn read_pmx_materials(mut stage: Self) -> (PMXMaterials, BonesLoader) {
        let mut ctx = PMXMaterials { materials: vec![] };
        let counts = stage.inner.read_i32();
        for _ in 0..counts {
            let material = stage.read_pmx_material();
            ctx.materials.push(material);
        }
        let next_stage = BonesLoader {
            header: stage.header,
            inner: stage.inner,
        };
        (ctx, next_stage)
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
            drawmode: 0,
            edge_color: [0.0f32; 4],
            edge_size: 0.0,
            texture_index: 0,
            sphere_mode_texture_index: 0,
            spheremode: PMXSphereMode::None,
            toon_mode: PMXToonMode::Separate,
            toon_texture_index: 0,
            memo: "".to_string(),
            num_face_vertices: 0,
        };
        ctx.name = self.inner.read_text_buf(self.header.encode);
        ctx.english_name = self.inner.read_text_buf(self.header.encode);
        ctx.diffuse = self.inner.read_vec4();
        ctx.specular = self.inner.read_vec3();
        ctx.specular_factor = self.inner.read_f32();
        ctx.ambient = self.inner.read_vec3();
        ctx.drawmode = self.inner.read_u8();
        ctx.edge_color = self.inner.read_vec4();
        ctx.edge_size = self.inner.read_f32();
        ctx.texture_index = self.inner.read_sized(s_texture_index).unwrap();
        ctx.sphere_mode_texture_index = self.inner.read_sized(s_texture_index).unwrap();
        let spmode = self.inner.read_u8();
        ctx.spheremode = match spmode {
            0 => PMXSphereMode::None,
            1 => PMXSphereMode::Mul,
            2 => PMXSphereMode::Add,
            3 => PMXSphereMode::SubTexture,
            _ => {
                panic!("Error Unknown SphereMode:{}", spmode);
            }
        };
        let toonmode = self.inner.read_u8();
        ctx.toon_mode = match toonmode {
            0 => PMXToonMode::Separate,
            1 => PMXToonMode::Common,
            _ => panic!("Error Unknown Toon flag:{}", toonmode),
        };
        ctx.toon_texture_index = match ctx.toon_mode {
            PMXToonMode::Separate => self.inner.read_sized(s_texture_index).unwrap(),
            PMXToonMode::Common => self.inner.read_u8() as i32,
        };
        ctx.memo = self.inner.read_text_buf(self.header.encode);
        ctx.num_face_vertices = self.inner.read_i32();
        ctx
    }
}
pub struct BonesLoader {
    header: PMXHeaderRust,
    inner: BinaryReader,
}
impl BonesLoader {
    pub fn get_header(&self) -> PMXHeaderRust {
        PMXHeaderRust {
            magic: "PMX ".to_string(),
            version: self.header.version,
            length: self.header.length,
            encode: self.header.encode,
            additional_uv: self.header.additional_uv,
            s_vertex_index: self.header.s_vertex_index,
            s_texture_index: self.header.s_texture_index,
            s_material_index: self.header.s_material_index,
            s_bone_index: self.header.s_bone_index,
            s_morph_index: self.header.s_morph_index,
            s_rigid_body_index: self.header.s_rigid_body_index,
        }
    }
    pub fn read_pmx_bones(mut stage: Self) -> (PMXBones, MorphsLoader) {
        let mut ctx = PMXBones { bones: vec![] };
        let count = stage.inner.read_i32();
        let mut v = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let bone = stage.read_pmx_bone();
            v.push(bone);
        }
        ctx.bones = v;

        let next_stage = MorphsLoader {
            header: stage.header,
            inner: stage.inner,
        };
        (ctx, next_stage)
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
            boneflag: 0,
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
        ctx.boneflag = self.inner.read_u16();
        //
        if (ctx.boneflag & BONE_FLAG_TARGET_SHOW_MODE_MASK) == BONE_FLAG_TARGET_SHOW_MODE_MASK {
            ctx.child = self.inner.read_sized(s_bone_index).unwrap();
        } else {
            ctx.offset = self.inner.read_vec3();
        }
        //Append rotate or Append translate
        if ctx.boneflag & (BONE_FLAG_APPEND_ROTATE_MASK | BONE_FLAG_APPEND_TRANSLATE_MASK) > 0 {
            ctx.append_bone_index = self.inner.read_sized(s_bone_index).unwrap();
            ctx.append_weight = self.inner.read_f32();
        }
        //Fixed Axis
        if (ctx.boneflag & BONE_FLAG_FIXED_AXIS_MASK) == BONE_FLAG_FIXED_AXIS_MASK {
            ctx.fixed_axis = self.inner.read_vec3();
        }
        //Local Axis
        if (ctx.boneflag & BONE_FLAG_LOCAL_AXIS_MASK) == BONE_FLAG_LOCAL_AXIS_MASK {
            ctx.local_axis_x = self.inner.read_vec3();
            ctx.local_axis_z = self.inner.read_vec3();
        }
        //outer deform
        if (ctx.boneflag & BONE_FLAG_DEFORM_OUTER_PARENT_MASK) > 0 {
            ctx.key_value = self.inner.read_i32();
        }
        //IK flag on
        if (ctx.boneflag & BONE_FLAG_IK_MASK) == BONE_FLAG_IK_MASK {
            ctx.ik_target_index = self.inner.read_sized(s_bone_index).unwrap();
            ctx.ik_iter_count = self.inner.read_i32();
            ctx.ik_limit = self.inner.read_f32();
            let ik_link_count = self.inner.read_i32();
            let mut ik_s = Vec::with_capacity(ik_link_count as usize);
            for _ in 0..ik_link_count {
                ik_s.push(self.read_iklink());
            }
            ctx.ik_links = ik_s;
            assert_eq!(ctx.ik_links.len(), ik_link_count as usize);
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

pub struct MorphsLoader {
    header: PMXHeaderRust,
    inner: BinaryReader,
}
impl MorphsLoader {
    pub fn get_header(&self) -> PMXHeaderRust {
        PMXHeaderRust {
            magic: "PMX ".to_string(),
            version: self.header.version,
            length: self.header.length,
            encode: self.header.encode,
            additional_uv: self.header.additional_uv,
            s_vertex_index: self.header.s_vertex_index,
            s_texture_index: self.header.s_texture_index,
            s_material_index: self.header.s_material_index,
            s_bone_index: self.header.s_bone_index,
            s_morph_index: self.header.s_morph_index,
            s_rigid_body_index: self.header.s_rigid_body_index,
        }
    }
    pub fn read_pmx_morphs(mut stage: Self) -> (PMXMorphs, ()) {
        let mut ctx = PMXMorphs { morphs: vec![] };
        let count = stage.inner.read_i32();
        let mut v = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let bone = stage.read_pmx_morph();
            v.push(bone);
        }
        ctx.morphs = v;

        (ctx, ())
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
