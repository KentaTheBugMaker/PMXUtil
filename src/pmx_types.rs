pub mod pmx_types {
    use std::fmt::{Display, Formatter};

    pub type Vec2 = [f32; 2];
    pub type Vec3 = [f32; 3];
    pub type Vec4 = [f32; 4];
    /// represent text encoding but all texts in pmx file are converted to String so you don't need to care
    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum Encode {
        UTF8 = 0x01,
        Utf16Le = 0x00,
    }
    /*Bone Flag*/
    pub const BONE_FLAG_TARGET_SHOW_MODE_MASK: u16 = 0x0001;
    pub const BONE_FLAG_ALLOW_ROTATE_MASK: u16 = 0x0002;
    //0b 0000 0000 0000 0010
    pub const BONE_FLAG_ALLOW_TRANSLATE_MASK: u16 = 0x0004;
    //0b 0000 0000 0000 0100
    pub const BONE_FLAG_VISIBLE_MASK: u16 = 0x0008;
    pub const BONE_FLAG_ALLOW_CONTROL_MASK: u16 = 0x0010;
    pub const BONE_FLAG_IK_MASK: u16 = 0x0020;
    pub const BONE_FLAG_APPEND_LOCAL_MASK: u16 = 0x0080;
    pub const BONE_FLAG_APPEND_ROTATE_MASK: u16 = 0x0100;
    //0b 0000 0001 0000 0000
    pub const BONE_FLAG_APPEND_TRANSLATE_MASK: u16 = 0x0200;
    //0b 0000 0010 0000 0000
    pub const BONE_FLAG_FIXED_AXIS_MASK: u16 = 0x0400;
    //0b 0000 0100 0000 0000
    pub const BONE_FLAG_LOCAL_AXIS_MASK: u16 = 0x0800;
    pub const BONE_FLAG_DEFORM_AFTER_PHYSICS_MASK: u16 = 0x1000;
    pub const BONE_FLAG_DEFORM_OUTER_PARENT_MASK: u16 = 0x2000;
    /*Material Flag*/
    const MATERIAL_DOUBLE_SIDE_MASK: u8 = 0x01;
    const MATERIAL_GROUND_SHADOW_MASK: u8 = 0x02;
    const MATERIAL_CAST_SELF_SHADOW_MASK: u8 = 0x04;
    const MATERIAL_RECEIVE_SELF_SHADOW_MASK: u8 = 0x08;
    const MATERIAL_EDGE_DRAW_MASK: u8 = 0x10;
    const MATERIAL_VERTEX_COLOR_MASK: u8 = 0x20;
    const MATERIAL_DRAW_POINT_MASK: u8 = 0x40;
    const MATERIAL_DRAW_LINE_MASK: u8 = 0x80;
    /// PMX仕様.txt 156~173
    #[repr(packed)]
    pub struct PMXHeaderC {
        pub magic: [u8; 4],
        pub version: f32,
        pub length: u8,
        pub config: [u8; 8],
    }
    ///these are pmx file header
    /// record magic number , version , text encoding ,and index size
    /// but internal use only so you don't need to care
    #[derive(Debug, Clone)]
    pub struct PMXHeaderRust {
        pub magic: String,
        pub version: f32,
        pub length: u8,
        pub encode: Encode,
        pub additional_uv: u8,
        pub s_vertex_index: u8,
        pub s_texture_index: u8,
        pub s_material_index: u8,
        pub s_bone_index: u8,
        pub s_morph_index: u8,
        pub s_rigid_body_index: u8,
    }
    ///represent index size of pmx data but these are converted to i32 or u32 so you don't need to care
    pub enum IndexSize {
        Byte,
        Short,
        Int,
    }
    /// these are pmx embedded comments and names
    /// PMX仕様.txt 176~181
    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct PMXModelInfo {
        pub name: String,
        pub name_en: String,
        pub comment: String,
        pub comment_en: String,
    }
    ///PMX仕様.txt 190~197
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum PMXVertexWeight {
        BDEF1(i32),
        BDEF2 {
            bone_index_1: i32,
            bone_index_2: i32,
            bone_weight_1: f32,
        },
        BDEF4 {
            bone_index_1: i32,
            bone_index_2: i32,
            bone_index_3: i32,
            bone_index_4: i32,
            bone_weight_1: f32,
            bone_weight_2: f32,
            bone_weight_3: f32,
            bone_weight_4: f32,
        },
        SDEF {
            bone_index_1: i32,
            bone_index_2: i32,
            bone_weight_1: f32,
            sdef_c: Vec3,
            sdef_r0: Vec3,
            sdef_r1: Vec3,
        },
        QDEF {
            bone_index_1: i32,
            bone_index_2: i32,
            bone_index_3: i32,
            bone_index_4: i32,
            bone_weight_1: f32,
            bone_weight_2: f32,
            bone_weight_3: f32,
            bone_weight_4: f32,
        },
    }
    ///these value are must submitted to vertex shader
    /// but bone_indices bone_weights must submitted to physics engine
    /// PMX仕様.txt 184~252
    #[derive(Debug, Clone, PartialEq)]
    pub struct PMXVertex {
        pub position: Vec3,
        pub norm: Vec3,
        pub uv: Vec2,
        pub add_uv: [Vec4; 4],
        pub weight_type: PMXVertexWeight,
        pub edge_mag: f32,
    }

    /*Represent Triangle*/
    /// represent one triangle
    /// PMX仕様.txt 255~257
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub struct PMXFace {
        pub vertices: [i32; 3],
    }
    /// texture file name list
    /// PMX仕様.txt 263~267
    #[derive(Debug, Eq, PartialEq)]
    pub struct PMXTextureList {
        pub textures: Vec<String>,
    }
    ///PMX仕様.txt 286~288
    pub enum PMXDrawModeFlags {
        BothFace = 0x01,
        GroundShadow = 0x02,
        CastSelfShadow = 0x04,
        RecieveSelfShadow = 0x08,
        DrawEdge = 0x10,
        VertexColor = 0x20,
        DrawPoint = 0x40,
        DrawLine = 0x80,
    }

    ///PMX仕様.txt 295
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum PMXSphereModeRaw {
        None = 0x00,
        Mul = 0x01,
        Add = 0x02,
        SubTexture = 0x03,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum PMXSphereMode {
        Mul(i32),
        Add(i32),
        SubTexture,
    }
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum PMXToonMode {
        Separate(i32),
        Common(i32),
    }

    ///PMX仕様.txt 297 ~ 303
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum PMXToonModeRaw {
        Separate = 0x00,
        //< 0:個別Toon
        Common = 0x01, //< 1:共有Toon[0-9] toon01.bmp～toon10.bmp
    }
    /// these values are must submitted to fragment or vertex shader by uniform or push_constant
    ///PMX仕様.txt 276~310
    #[derive(Debug, Clone, PartialEq)]
    pub struct PMXMaterial {
        pub name: String,
        pub english_name: String,
        pub diffuse: Vec4,
        pub specular: Vec3,
        pub specular_factor: f32,
        pub ambient: Vec3,
        pub draw_mode: u8,
        pub edge_color: Vec4,
        pub edge_size: f32,
        pub texture_index: i32,
        pub sphere_mode_texture_index: i32,
        pub sphere_mode: PMXSphereModeRaw,
        pub toon_mode: PMXToonModeRaw,
        pub toon_texture_index: i32,
        pub memo: String,
        pub num_face_vertices: i32,
    }
    ///PMX仕様.txt 476~497
    #[derive(Debug, Clone, PartialEq)]
    pub struct PMXFrame {
        pub name: String,
        pub name_en: String,
        pub is_special: u8,
        pub inners: Vec<FrameInner>,
    }
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct FrameInner {
        pub target: u8,
        pub index: i32,
    }
    ///PMX仕様.txt 313 ~395
    #[derive(Debug, Clone, PartialEq)]
    pub struct PMXBone {
        pub name: String,
        pub english_name: String,
        pub position: Vec3,
        pub parent: i32,
        pub deform_depth: i32,
        pub boneflag: u16,
        pub offset: Vec3,
        pub child: i32,
        pub append_bone_index: i32,
        pub append_weight: f32,
        pub fixed_axis: Vec3,
        pub local_axis_x: Vec3,
        pub local_axis_z: Vec3,
        pub key_value: i32,
        pub ik_target_index: i32,
        pub ik_iter_count: i32,
        pub ik_limit: f32,
        pub ik_links: Vec<PMXIKLink>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct PMXIKLink {
        pub ik_bone_index: i32,
        pub enable_limit: u8,
        pub limit_min: Vec3,
        pub limit_max: Vec3,
    }
    ///PMX仕様.txt 399~459
    #[derive(Debug, Clone, PartialEq)]
    pub struct PMXMorph {
        pub name: String,
        pub english_name: String,
        pub category: u8,
        pub morph_type: u8,
        pub offset: i32,
        pub morph_data: Vec<MorphTypes>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum MorphTypes {
        Vertex(VertexMorph),
        UV(UVMorph),
        UV1(UVMorph),
        UV2(UVMorph),
        UV3(UVMorph),
        UV4(UVMorph),
        Bone(BoneMorph),
        Material(MaterialMorph),
        Group(GroupMorph),
        Flip(FlipMorph),
        Impulse(ImpulseMorph),
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct VertexMorph {
        pub index: i32,
        pub offset: Vec3,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct UVMorph {
        pub index: i32,
        pub offset: Vec4,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct GroupMorph {
        pub index: i32,
        pub morph_factor: f32,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct BoneMorph {
        pub index: i32,
        pub translates: Vec3,
        pub rotates: Vec4,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct MaterialMorph {
        pub index: i32,
        pub formula: u8,
        pub diffuse: Vec4,
        pub specular: Vec3,
        pub specular_factor: f32,
        pub ambient: Vec3,
        pub edge_color: Vec4,
        pub edge_size: f32,
        pub texture_factor: Vec4,
        pub sphere_texture_factor: Vec4,
        pub toon_texture_factor: Vec4,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct FlipMorph {
        pub index: i32,
        pub morph_factor: f32,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ImpulseMorph {
        pub rigid_index: i32,
        pub is_local: u8,
        pub velocity: Vec3,
        pub torque: Vec3,
    }
    #[derive(Debug,Clone)]
    pub struct PMXRigid {
        pub name: String,
        pub name_en: String,
        pub bone_index: i32,
        pub group: u8,
        pub un_collision_group_flag: u16,
        pub form: PMXRigidForm,
        pub size: Vec3,
        pub position: Vec3,
        pub rotation: Vec3,
        pub mass: f32,
        pub move_resist: f32,
        pub rotation_resist: f32,
        pub repulsion: f32,
        pub friction: f32,
        pub calc_method: PMXRigidCalcMethod,
    }

    #[derive(Clone,Debug)]
    pub enum PMXRigidForm {
        Sphere,
        //0
        Box,
        //1
        Capsule, //2
    }

    #[derive(Debug, Clone)]
    pub enum PMXRigidCalcMethod {
        Static,
        //0
        Dynamic,
        //1
        DynamicWithBonePosition, //2
    }

    /// C bridge
    /// This struct is fixed size so we can use read_bin!
    #[repr(packed)]
    pub(crate) struct PMXJointParameterRaw {
        pub(crate) joint_type: u8,
        pub(crate) a_rigid_index: i32,
        pub(crate) b_rigid_index: i32,
        pub(crate) position: Vec3,
        pub(crate) rotation: Vec3,
        pub(crate) move_limit_down: Vec3,
        pub(crate) move_limit_up: Vec3,
        pub(crate) rotation_limit_down: Vec3,
        pub(crate) rotation_limit_up: Vec3,
        pub(crate) spring_const_move: Vec3,
        pub(crate) spring_const_rotation: Vec3,
    }

    #[derive(Clone)]
    pub struct PMXJoint {
        pub name: String,
        pub name_en: String,
        pub joint_type: PMXJointType,
    }

    #[derive(Clone)]
    pub enum PMXJointType {
        ///Support from PMXUtil 0.4.0
        Spring6DOF {
            a_rigid_index: i32,
            b_rigid_index: i32,
            position: Vec3,
            rotation: Vec3,
            move_limit_down: Vec3,
            move_limit_up: Vec3,
            rotation_limit_down: Vec3,
            rotation_limit_up: Vec3,
            spring_const_move: Vec3,
            spring_const_rotation: Vec3,
        },
        _6DOF {
            a_rigid_index: i32,
            b_rigid_index: i32,
            position: Vec3,
            rotation: Vec3,
            move_limit_down: Vec3,
            move_limit_up: Vec3,
            rotation_limit_down: Vec3,
            rotation_limit_up: Vec3,
        },
        P2P {
            a_rigid_index: i32,
            b_rigid_index: i32,
            position: Vec3,
            rotation: Vec3,
        },
        ConeTwist {
            a_rigid_index: i32,
            b_rigid_index: i32,
            swing_span1: f32,
            swing_span2: f32,
            twist_span: f32,
            softness: f32,
            bias_factor: f32,
            relaxation_factor: f32,
            damping: f32,
            fix_thresh: f32,
            enable_motor: bool,
            max_motor_impulse: f32,
            motor_target_in_constraint_space: Vec3,
        },
        Slider {
            a_rigid_index: i32,
            b_rigid_index: i32,
            lower_linear_limit: f32,
            upper_linear_limit: f32,
            lower_angle_limit: f32,
            upper_angle_limit: f32,
            power_linear_motor: bool,
            target_linear_motor_velocity: f32,
            max_linear_motor_force: f32,
            power_angler_motor: bool,
            target_angler_motor_velocity: f32,
            max_angler_motor_force: f32,
        },
        Hinge {
            a_rigid_index: i32,
            b_rigid_index: i32,
            low: f32,
            high: f32,
            softness: f32,
            bias_factor: f32,
            relaxation_factor: f32,
            enable_motor: bool,

            target_velocity: f32,
            max_motor_impulse: f32,
        },
    }
    /// from PMXUtil 0.5.0
    pub struct PMXSoftBody {
        pub name: String,
        pub name_en: String,
        pub form: PMXSoftBodyForm, //i8
        pub material_index: i32,
        pub group: u8,
        pub un_collision_group_flag: u16,
        pub bit_flag: u8,
        pub b_link_create_distance: i32,
        pub clusters: i32,
        pub mass: f32,
        pub collision_margin: f32,
        pub aero_model: PMXSoftBodyAeroModel, //i32
        ///config
        pub vcf: f32,
        pub dp: f32,
        pub dg: f32,
        pub lf: f32,
        pub pr: f32,
        pub vc: f32,
        pub df: f32,
        pub mt: f32,
        pub chr: f32,
        pub khr: f32,
        pub shr: f32,
        pub ahr: f32,
        ///cluster
        pub srhr_cl: f32,
        pub skhr_cl: f32,
        pub sshr_cl: f32,
        pub sr_splt_cl: f32,
        pub sk_splt_cl: f32,
        pub ss_splt_cl: f32,
        ///iteration
        pub v_it: i32,
        pub p_it: i32,
        pub d_it: i32,
        pub c_it: i32,
        ///material
        pub lst: f32,
        pub ast: f32,
        pub vst: f32,
        pub anchor_rigid: Vec<PMXSoftBodyAnchorRigid>,
        pub pin_vertex: Vec<i32>,
    }
    pub struct PMXSoftBodyAnchorRigid {
        pub rigid_index: i32,
        pub vertex_index: i32,
        pub near_mode: bool,
    }

    pub enum PMXSoftBodyForm {
        TriMesh,
        Rope,
    }
    pub enum PMXSoftBodyAeroModel {
        VPoint,
        VTwoSide,
        VOneSided,
        FTwoSided,
        FOneSided,
    }

    impl Display for PMXVertex {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            writeln!(
                f,
                "Vertex:[position:{:?} norm:{:?} uv:{:?}]",
                self.position, self.norm, self.uv
            );
            if [[0.0f32; 4]; 4] != self.add_uv {
                for add_uv in &self.add_uv {
                    write!(f, "{:?}", add_uv);
                }
            }
            match self.weight_type {
                PMXVertexWeight::BDEF1(b1) => {
                    writeln!(f, "BDEF1:[index1:{} weight1:1.0]", b1);
                }
                PMXVertexWeight::BDEF2 {
                    bone_index_1,
                    bone_index_2,
                    bone_weight_1,
                } => {
                    writeln!(
                        f,
                        "BDEF2:[index1:{} index2:{} weight1:{} weight2:{}]",
                        bone_index_1,
                        bone_index_2,
                        bone_weight_1,
                        1.0 - bone_weight_1
                    );
                }
                PMXVertexWeight::BDEF4 {
                    bone_index_1,
                    bone_index_2,
                    bone_index_3,
                    bone_index_4,
                    bone_weight_1,
                    bone_weight_2,
                    bone_weight_3,
                    bone_weight_4,
                } => {
                    writeln!(f, "BDEF4:[index1:{} index2:{} index3:{} index4:{}, weight1:{} weight2:{} weight3:{} weight4:{}]", bone_index_1, bone_index_2, bone_index_3, bone_index_4, bone_weight_1, bone_weight_2, bone_weight_3, bone_weight_4);
                }
                PMXVertexWeight::SDEF {
                    bone_index_1,
                    bone_index_2,
                    bone_weight_1,
                    sdef_c,
                    sdef_r0,
                    sdef_r1,
                } => {
                    writeln!(
                        f,
                        "SDEF:[index1:{} index2:{} weight1:{} weight2:{}]",
                        bone_index_1,
                        bone_index_2,
                        bone_weight_1,
                        1.0 - bone_weight_1
                    );
                    writeln!(
                        f,
                        "SDEF Specific Params:[ C:[{:?}] R0:[{:?}] R1:[{:?}] ]",
                        sdef_c, sdef_r0, sdef_r1
                    );
                }
                PMXVertexWeight::QDEF {
                    bone_index_1,
                    bone_index_2,
                    bone_index_3,
                    bone_index_4,
                    bone_weight_1,
                    bone_weight_2,
                    bone_weight_3,
                    bone_weight_4,
                } => {
                    writeln!(f, "BDEF4:[index1:{} index2:{} index3:{} index4:{}, weight1:{} weight2:{} weight3:{} weight4:{}]", bone_index_1, bone_index_2, bone_index_3, bone_index_4, bone_weight_1, bone_weight_2, bone_weight_3, bone_weight_4);
                }
            }
            writeln!(f, "edgeMagnifier:{}", self.edge_mag);
            Ok(())
        }
    }

    impl Display for PMXFace {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            writeln!(
                f,
                "Triangle:[{},{},{}]",
                self.vertices[0], self.vertices[1], self.vertices[2]
            );
            Ok(())
        }
    }

    impl Display for PMXTextureList {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            let _result = writeln!(f, "Textures:{}", self.textures.len());
            for name in self.textures.iter() {
                let _result = writeln!(f, "{}", name);
            }
            Ok(())
        }
    }
}
