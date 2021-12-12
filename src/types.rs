//! PMX type definitions.

use bitflags::bitflags;
use std::convert::TryFrom;

pub type Vec2 = [f32; 2];
pub type Vec3 = [f32; 3];
pub type Vec4 = [f32; 4];

/// represent text encoding but all texts in pmx file are converted to String so you don't need to care
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Encode {
    UTF8 = 0x01,
    Utf16Le = 0x00,
}

/// PMX仕様.txt 156~173
#[repr(packed)]
pub struct HeaderRaw {
    pub magic: [u8; 4],
    pub version: f32,
    pub length: u8,
    pub config: [u8; 8],
}

#[derive(Debug, Clone)]
pub enum PMXVersion {
    V20,
    V21,
}

/// rustic wrapped header.
#[derive(Debug, Clone)]
pub struct Header {
    pub(crate) magic: String,
    pub version: PMXVersion,
    pub(crate) length: u8,
    pub encode: Encode,
    pub additional_uv: u8,
    pub(crate) s_vertex_index: VertexIndexKinds,
    pub(crate) s_texture_index: IndexKinds,
    pub(crate) s_material_index: IndexKinds,
    pub(crate) s_bone_index: IndexKinds,
    pub(crate) s_morph_index: IndexKinds,
    pub(crate) s_rigid_body_index: IndexKinds,
}

/// Pmx embedded comments and names
///
/// refer PMX仕様.txt 176~181
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub name_en: String,
    pub comment: String,
    pub comment_en: String,
}

/// Defining how to calculate skinning.
///
/// refer PMX仕様.txt 190~197
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VertexWeight {
    /// a bone with weight 1.0
    BDEF1(i32),
    /// 2 bones with normalized weight
    /// * bone_weight_1 : weight of bone_index_1
    /// * bone_weight_2 : 1.0 - bone_weight_1
    BDEF2 {
        bone_index_1: i32,
        bone_index_2: i32,
        bone_weight_1: f32,
    },
    /// 4 bones without normalized weights guaranty.
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
    /// spherical deforming bones
    ///
    /// We can't find official `SDEF` code but maybe this code usable.
    /// ``` hlsl
    /// //    影響度算出
    /// void CalcSdefWeight( out float _rWeight0,
    ///                      out float _rWeight1,
    ///                       in float3 _rSdefR0,
    ///                       in float3 _rSdefR1 )
    /// {
    ///     float    l0    = length( _rSdefR0 );
    ///     float    l1    = length( _rSdefR1 );
    ///     if( abs( l0 - l1 ) < 0.0001f )
    ///     {
    ///         _rWeight1    = 0.5f;
    ///     }
    ///     else
    ///     {
    ///         _rWeight1    = saturate( l0 / ( l0 + l1 ) );
    ///     }
    ///     _rWeight0    = 1.0f - _rWeight1;
    /// }
    ///
    /// //    SDEF コード
    /// {
    ///     int     b0    = _stIn.BIndex[0],
    ///             b1    = _stIn.BIndex[1];
    ///     float   w0    = _stIn.BWeight[0],
    ///             w1    = _stIn.BWeight[1];
    ///     //    先に影響度を算出する
    ///     float   w2, w3;
    ///     CalcSdefWeight( w2, w3, _stIn.SdefR0 + _stIn.SdefC, _stIn.SdefR1 + _stIn.SdefC );
    ///     //    C点を算出する
    ///     float4   r0   = float4( _stIn.SdefR0 + _stIn.SdefC, 1 );
    ///     float4   r1   = float4( _stIn.SdefR1 + _stIn.SdefC, 1 );
    ///     matrix   m0   = m_mPMXBoneMatrix[b0];
    ///     matrix   m1   = m_mPMXBoneMatrix[b1];
    ///     matrix   mrc  = m0 * w0 + m1 * w1;
    ///     float3   prc  = mul( float4( _stIn.SdefC, 1 ), mrc ).xyz;
    ///     //    r0, r1による差分値を算出して加算
    ///     {
    ///         matrix    m2    = m0 * w0;
    ///         matrix    m3    = m1 * w1;
    ///         matrix    m     = m2 + m3;
    ///         float3    v0    = mul( r0,m2 + m * w1 ).xyz - mul( r0, m ).xyz;
    ///         float3    v1    = mul( r1,m * w0 + m3 ).xyz - mul( r1, m ).xyz;
    ///         prc            += v0 * w2 + v1 * w3;
    ///     }
    ///     //    回転して加算
    ///     float4    q0  = m_vPMXBoneQuat[b0] * w0;
    ///     float4    q1  = m_vPMXBoneQuat[b1] * w1;
    ///     matrix    m   = QuaternionToMatrix( slerp( q0, q1, w3 ) );
    ///     _vPos         = prc + mul( float4( _vPos - _stIn.SdefC, 1 ), m ).xyz;
    ///     _vNorm        = mul( float4( _vNorm.xyz, 1 ), m ).xyz;
    /// }
    /// ```
    SDEF {
        bone_index_1: i32,
        bone_index_2: i32,
        bone_weight_1: f32,
        sdef_c: Vec3,
        sdef_r0: Vec3,
        sdef_r1: Vec3,
    },
    /// DualQuaternion deforming
    ///
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
/// PMX仕様.txt 184~252
///
/// you can pass by below codes
/// ``` glsl
///  // per vertex
///  layout(location = 0) in vec3 a_pos;
///  layout(location = 1) in vec3 a_normal;
///  layout(location = 2) in vec2 a_tex_coord;
///  layout(location = 3) in vec4 add_uv1;
///  layout(location = 4) in vec4 add_uv2;
///  layout(location = 5) in vec4 add_uv3;
///  layout(location = 6) in vec4 add_uv4;
///  // bone and weights
///  layout(location = 7) in int bone_kind;
///  layout(location = 8) in vec4 bone_weights;
///  layout(location = 9) in ivec4 bone_indices;
///  // edge magnifier
///  layout(location = 10) in float edge_mag;
///  ```
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub norm: Vec3,
    pub uv: Vec2,
    pub add_uv: [Vec4; 4],
    pub weight_type: VertexWeight,
    pub edge_mag: f32,
}

/// In PMX 2.0 represent one triangle but PMX 2.1 you need to determine drawing primitive.
///
/// # How to determine primitives In  2.1
/// * `TriangleList` if
/// `!Material.draw_mode.intersects(MaterialFlags::POINT_DRAW|MaterialFlag::LINE_DRAW)`
/// * `LineList` if
/// `Material.draw_mode.intersects(MaterialFlags::LINE_DRAW) && !Material.draw_mode.intersects(MaterialFlag::POINT_DRAW) `
/// * `PointList` if
/// `Material.draw_mode.intersects(MaterialFlags::POINT_DRAW)`
///
/// # Recording format of each primitive
/// * `TriangleList`
///     A-B-C
/// * `LineList`
///     A-B-A
///     so you can drop last point without any problems in this face
/// * `Point`
///     A-A-A
///     so you only need to pass first vertex index in this face
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Face {
    pub vertices: [i32; 3],
}
/// texture file name list
///
/// refer PMX仕様.txt 263~273
/// relative path from pmx file located directory
///
/// path separator may contains `/` or `\` so unix-like system will need  to convert it
///
#[derive(Debug, Eq, PartialEq)]
pub struct TextureList {
    pub textures: Vec<String>,
}
/// how to apply sphere mode texture
/// refer PMX仕様.txt 295
///
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SphereModeKind {
    Mul,
    Add,
    SubTexture,
}

/// ergonomic sphere mode representation in Rust
///
/// let see [`SphereModeKind`](crate::types::SphereModeKind)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SphereMode {
    pub index: i32,
    pub kind: SphereModeKind,
}
/// represent which texture need to use for toon
/// * Separate use texture in texture list
/// * Common use embedded  texture in MMD or `PMXe`
/// refer  PMX仕様.txt 297 ~ 303
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ToonMode {
    Separate(i32),
    Common(u8),
}

///
///
///
/// as a sample you can pass Vulkan like glsl code.
/// ``` glsl
/// layout (set = 0 ,binding = 0) Material{
///     vec4 diffuse;
///     vec3 specular;
///     float specular_exponent;
///     vec3 ambient;
///     bool render_self_shadow;
///     vec4 edge_color;
///     int sphere_texture_mode;
/// }
/// layout (set = 1 ,binding = 0) sampler2D color_texture;
/// layout (set = 1 ,binding = 1) sampler2D sphere_texture;
/// layout (set = 1 ,binding = 2) sampler2D toon_texture;
/// ```
///
///  refer PMX仕様.txt 276~310
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    pub english_name: String,
    pub diffuse: Vec4,
    pub specular: Vec3,
    pub specular_factor: f32,
    pub ambient: Vec3,
    pub draw_mode: MaterialFlags,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub texture_index: i32,
    pub sphere_mode: Option<SphereMode>,
    pub toon_mode: ToonMode,
    pub memo: String,
    pub num_face_vertices: i32,
}
///from PMX仕様.txt 476 ~ 497
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub name: String,
    pub name_en: String,
    pub is_special: u8,
    pub inners: Vec<FrameInner>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FrameInner {
    pub target: Target,
    pub index: i32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Target {
    Bone,
    Morph,
}

///refer PMX仕様.txt 348 ~ 354
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConnectionDisplayMode {
    OtherBone(i32),
    Offset(Vec3),
}
impl Default for ConnectionDisplayMode {
    fn default() -> Self {
        Self::OtherBone(-1)
    }
}
/// represent one bone
///
/// vb
///
/// refer PMX仕様.txt 313 ~ 395
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Bone {
    pub name: String,
    pub english_name: String,
    pub position: Vec3,
    pub parent: i32,
    pub deform_depth: i32,
    /// bone flag 0x0001
    pub connection_display_mode: ConnectionDisplayMode,
    /// bone flag0x0002
    pub rotatable_in_viewer: bool,
    /// bone flag 0x0004
    pub translatable_in_viewer: bool,
    /// bone flag 0x0008
    pub display_bone_in_viewer: bool,
    /// bone flag 0x0010
    pub controllable_in_viewer: bool,
    /// bone flag 0x0080 0x0100 0x0200
    pub inherits: BoneInherits,
    /// 0x0400 refer PMX仕様.txt 362 ~ 365
    pub fixed_axis: Option<Vec3>,
    /// 0x0800 refer PMX仕様.txt 367 ~ 371
    pub local_axis: Option<(Vec3, Vec3)>,
    /// 0x1000
    pub physics_after_deform: bool,
    /// 0x2000 refer PMX仕様.txt 373 ~ 376
    pub external_parent: Option<i32>,
    /// 0x0020 refer PMX仕様.txt 378 ~ 396
    pub ik_info: Option<BoneIKInfo>,
}
impl Bone {
    pub fn calculate_bone_flag(&self) -> BoneFlags {
        let mut flags = BoneFlags::empty();
        //0x0001
        flags |= match self.connection_display_mode {
            ConnectionDisplayMode::OtherBone(_) => BoneFlags::CONNECT_TO_OTHER_BONE,
            ConnectionDisplayMode::Offset(_) => BoneFlags::empty(),
        };
        flags |= if self.rotatable_in_viewer {
            BoneFlags::ROTATABLE
        } else {
            BoneFlags::empty()
        };
        flags |= if self.translatable_in_viewer {
            BoneFlags::TRANSLATABLE
        } else {
            BoneFlags::empty()
        };
        flags |= if self.display_bone_in_viewer {
            BoneFlags::IS_VISIBLE
        } else {
            BoneFlags::empty()
        };
        flags |= if self.controllable_in_viewer {
            BoneFlags::ENABLED
        } else {
            BoneFlags::empty()
        };

        //0x0020
        flags |= if self.ik_info.is_some() {
            BoneFlags::IK
        } else {
            BoneFlags::empty()
        };
        flags |= if self.inherits.inherit_local {
            BoneFlags::INHERIT_LOCAL
        } else {
            BoneFlags::empty()
        };
        flags |= match self.inherits.rotate_and_translate {
            RotateAndTranslateInherits::None | RotateAndTranslateInherits::Translate(_, _) => {
                BoneFlags::empty()
            }
            RotateAndTranslateInherits::Both(_, _) | RotateAndTranslateInherits::Rotate(_, _) => {
                BoneFlags::INHERIT_ROTATION
            }
        };
        flags |= match self.inherits.rotate_and_translate {
            RotateAndTranslateInherits::None | RotateAndTranslateInherits::Rotate(_, _) => {
                BoneFlags::empty()
            }
            RotateAndTranslateInherits::Both(_, _)
            | RotateAndTranslateInherits::Translate(_, _) => BoneFlags::INHERIT_TRANSLATION,
        };
        flags |= if self.fixed_axis.is_some() {
            BoneFlags::FIXED_AXIS
        } else {
            BoneFlags::empty()
        };
        flags |= if self.local_axis.is_some() {
            BoneFlags::LOCAL_COORDINATE
        } else {
            BoneFlags::empty()
        };
        flags |= if self.physics_after_deform {
            BoneFlags::PHYSICS_AFTER_DEFORM
        } else {
            BoneFlags::empty()
        };
        flags |= if self.external_parent.is_some() {
            BoneFlags::EXTERNAL_PARENT_DEFORM
        } else {
            BoneFlags::empty()
        };
        flags
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct BoneInherits {
    pub inherit_local: bool,
    pub rotate_and_translate: RotateAndTranslateInherits,
}

/// represents how inherits rotate and translate
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RotateAndTranslateInherits {
    None,
    Both(i32, f32),
    Rotate(i32, f32),
    Translate(i32, f32),
}
impl Default for RotateAndTranslateInherits {
    fn default() -> Self {
        Self::None
    }
}

/// refer PMX仕様.txt 378 ~ 396
#[derive(Debug, Clone, PartialEq)]
pub struct BoneIKInfo {
    /// refer PMX仕様.txt 381
    pub ik_target_bone_index: i32,
    /// refer PMX仕様.txt 382
    pub ik_iter_count: i32,
    /// refer PMX仕様.txt 383
    pub ik_limit_angle: f32,
    /// refer PMX仕様.txt 385 ~ 395
    pub ik_links: Vec<IKLink>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IKLink {
    pub ik_bone_index: i32,
    pub angle_limit: Option<(Vec3, Vec3)>,
}
///PMX仕様.txt 399~459
#[derive(Debug, Clone, PartialEq)]
pub struct Morph {
    pub name: String,
    pub english_name: String,
    pub control_panel: ControlPanel,
    pub morph_data: MorphKinds,
}

/// where to place morph.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum ControlPanel {
    BottomLeft,
    TopLeft,
    TopRight,
    BottomRight,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MorphKinds {
    Vertex(Vec<VertexMorph>),
    UV(Vec<UVMorph>),
    UV1(Vec<UVMorph>),
    UV2(Vec<UVMorph>),
    UV3(Vec<UVMorph>),
    UV4(Vec<UVMorph>),
    Bone(Vec<BoneMorph>),
    Material(Vec<MaterialMorph>),
    Group(Vec<GroupMorph>),
    Flip(Vec<FlipMorph>),
    Impulse(Vec<ImpulseMorph>),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Rigid {
    pub name: String,
    pub name_en: String,
    pub bone_index: i32,
    pub group: u8,
    pub un_collision_group_flag: u16,
    pub form: RigidForm,
    pub size: Vec3,
    pub position: Vec3,
    pub rotation: Vec3,
    pub mass: f32,
    pub move_resist: f32,
    pub rotation_resist: f32,
    pub repulsion: f32,
    pub friction: f32,
    pub calc_method: RigidCalcMethod,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RigidForm {
    Sphere,
    //0
    Box,
    //1
    Capsule, //2
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RigidCalcMethod {
    Static,
    //0
    Dynamic,
    //1
    DynamicWithBonePosition, //2
}

/// C bridge
#[repr(packed)]
pub(crate) struct JointParameterRaw {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Joint {
    pub name: String,
    pub name_en: String,
    pub joint_type: JointType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JointType {
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
    SixDof {
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
/// from Util 0.5.0
#[derive(Debug, Clone)]
pub struct SoftBody {
    pub name: String,
    pub name_en: String,
    pub form: SoftBodyForm, //i8
    pub material_index: i32,
    pub group: u8,
    pub un_collision_group_flag: u16,
    pub bit_flag: u8,
    pub b_link_create_distance: i32,
    pub clusters: i32,
    pub mass: f32,
    pub collision_margin: f32,
    pub aero_model: SoftBodyAeroModel, //i32
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
    pub anchor_rigid: Vec<SoftBodyAnchorRigid>,
    pub pin_vertex: Vec<i32>,
}
#[derive(Debug, Copy, Clone)]
pub struct SoftBodyAnchorRigid {
    pub rigid_index: i32,
    pub vertex_index: i32,
    pub near_mode: bool,
}
#[derive(Debug, Copy, Clone)]
pub enum SoftBodyForm {
    TriMesh,
    Rope,
}
#[derive(Debug, Copy, Clone)]
pub enum SoftBodyAeroModel {
    VPoint,
    VTwoSide,
    VOneSided,
    FTwoSided,
    FOneSided,
}

bitflags! {
    pub struct MaterialFlags :u8 {
        const DISABLE_CULLING =0x01;
        const GROUND_SHADOW =0x02;
        const DRAW_SHADOW =0x04;
        const RECEIVE_SHADOW= 0x08;
        const HAS_EDGE =0x10;
        const VERTEX_COLOR = 0x20;
        const POINT_DRAW = 0x40;
        const LINE_DRAW =  0x80;
    }
}

bitflags! {
    pub struct BoneFlags : u16{
        const CONNECT_TO_OTHER_BONE=0x01;
        const ROTATABLE =0x02;
        const TRANSLATABLE =0x04;
        const IS_VISIBLE =0x08;
        const ENABLED = 0x10;
        const IK = 0x20;
        const INHERIT_LOCAL = 0x80;
        const INHERIT_ROTATION =0x100;
        const INHERIT_TRANSLATION = 0x200;
        const FIXED_AXIS = 0x400;
        const LOCAL_COORDINATE = 0x800;
        const PHYSICS_AFTER_DEFORM = 0x1000;
        const EXTERNAL_PARENT_DEFORM = 0x2000;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IndexKinds {
    I8,
    I16,
    I32,
}

#[derive(Clone, Copy, Debug)]
pub enum VertexIndexKinds {
    U8,
    U16,
    I32,
}

impl TryFrom<u8> for IndexKinds {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::I8),
            2 => Ok(Self::I16),
            4 => Ok(Self::I32),
            _ => Err(()),
        }
    }
}

impl From<IndexKinds> for u8 {
    fn from(kind: IndexKinds) -> Self {
        match kind {
            IndexKinds::I8 => 1,
            IndexKinds::I16 => 2,
            IndexKinds::I32 => 4,
        }
    }
}

impl TryFrom<u8> for VertexIndexKinds {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::U8),
            2 => Ok(Self::U16),
            4 => Ok(Self::I32),
            _ => Err(()),
        }
    }
}
impl From<VertexIndexKinds> for u8 {
    fn from(kind: VertexIndexKinds) -> Self {
        match kind {
            VertexIndexKinds::U8 => 1,
            VertexIndexKinds::U16 => 2,
            VertexIndexKinds::I32 => 4,
        }
    }
}
#[derive(Debug)]
pub enum HeaderConversionError {
    InvalidMagic,
    InvalidEncoding,
    InvalidIndex,
    InvalidVersion,
}
