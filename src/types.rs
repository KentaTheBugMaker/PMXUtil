use bitflags::bitflags;
use std::convert::TryFrom;

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

/// 仕様.txt 156~173
#[repr(packed)]
pub struct HeaderRaw {
    pub magic: [u8; 4],
    pub version: f32,
    pub length: u8,
    pub config: [u8; 8],
}

///these are pmx file header
/// record magic number , version , text encoding ,and index size
/// but internal use only so you don't need to care
#[derive(Debug, Clone)]
pub struct Header {
    pub magic: String,
    pub version: f32,
    pub length: u8,
    pub encode: Encode,
    pub additional_uv: u8,
    pub s_vertex_index: VertexIndexKinds,
    pub s_texture_index: IndexKinds,
    pub s_material_index: IndexKinds,
    pub s_bone_index: IndexKinds,
    pub s_morph_index: IndexKinds,
    pub s_rigid_body_index: IndexKinds,
}

/// these are pmx embedded comments and names
/// 仕様.txt 176~181
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub name_en: String,
    pub comment: String,
    pub comment_en: String,
}

///仕様.txt 190~197
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VertexWeight {
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
/// 仕様.txt 184~252
///
/// you can pass by below codes
/// ```glsl
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
/// 仕様.txt 263~267
#[derive(Debug, Eq, PartialEq)]
pub struct TextureList {
    pub textures: Vec<String>,
}

///仕様.txt 295
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SphereModeKind {
    Mul,
    Add,
    SubTexture,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SphereMode {
    pub(crate) index: i32,
    pub(crate) kind: SphereModeKind,
}

///仕様.txt 297 ~ 303
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ToonMode {
    Separate(i32),
    Common(u8),
}

/// these values are must submitted to fragment or vertex shader by uniform or `push_constant`
///仕様.txt 276~310
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
///仕様.txt 476~497
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
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
///仕様.txt 313 ~395
#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    pub name: String,
    pub english_name: String,
    pub position: Vec3,
    pub parent: i32,
    pub deform_depth: i32,
    pub boneflag: BoneFlags,
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
    pub ik_links: Vec<IKLink>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IKLink {
    pub ik_bone_index: i32,
    pub enable_limit: u8,
    pub limit_min: Vec3,
    pub limit_max: Vec3,
}
///仕様.txt 399~459
#[derive(Debug, Clone, PartialEq)]
pub struct Morph {
    pub name: String,
    pub english_name: String,
    pub category: u8,
    pub morph_type: u8,
    pub offset: i32,
    pub morph_data: Vec<MorphKinds>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MorphKinds {
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
    ///Support from Util 0.4.0
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
#[derive(Debug)]
pub enum HeaderConversionError {
    InvalidMagic,
    InvalidEncoding,
    InvalidIndex,
}
