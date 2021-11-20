use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::Write;
use std::mem::transmute;
use std::path::Path;

use crate::pmx_types::{
    BoneMorph, FlipMorph, GroupMorph, ImpulseMorph, MaterialMorph, MorphTypes, PMXBone, PMXFace,
    PMXFrame, PMXIKLink, PMXJoint, PMXJointType, PMXMaterial, PMXMorph, PMXRigid,
    PMXRigidCalcMethod, PMXRigidForm, PMXSphereModeRaw, PMXToonModeRaw, PMXVertex, PMXVertexWeight,
    UVMorph, VertexMorph, BONE_FLAG_APPEND_ROTATE_MASK, BONE_FLAG_APPEND_TRANSLATE_MASK,
    BONE_FLAG_DEFORM_OUTER_PARENT_MASK, BONE_FLAG_FIXED_AXIS_MASK, BONE_FLAG_IK_MASK,
    BONE_FLAG_LOCAL_AXIS_MASK, BONE_FLAG_TARGET_SHOW_MODE_MASK,
};
use crate::pmx_types::{Vec2, Vec3, Vec4};

/// This is internal use only struct
/// Do not use this struct
pub(crate) struct BinaryWriter {
    pub(crate) inner: BufWriter<File>,
    pub(crate) is_utf16: bool,
}
macro_rules! write_bin {
    ($F:ident,$T:ty) => {
        ///Macro implemented member for internal use
        pub(crate) fn $F(&mut self, value: $T) {
            let mut buf = [0u8; std::mem::size_of::<$T>()];
            unsafe { buf = transmute(value) };
            self.inner.write_all(&buf).unwrap();
        }
    };
}
impl BinaryWriter {
    pub fn create<P: AsRef<Path>>(path: P, is_utf16: bool) -> Result<BinaryWriter, Error> {
        //   let file = File::open(&path);
        let file = File::create(&path);
        let file_size = std::fs::metadata(&path).unwrap().len();

        match file {
            Ok(file) => {
                let inner = BufWriter::with_capacity(file_size as usize, file);
                Ok(BinaryWriter { inner, is_utf16 })
            }
            Err(err) => Err(err),
        }
    }
    pub(crate) fn write_vec(&mut self, v: &[u8]) {
        self.inner.write_all(v).unwrap();
    }

    pub(crate) fn write_text_buf(&mut self, text: &str) {
        let len = text.len();
        if self.is_utf16 {
            let bytes=encoding_rs::UTF_16LE.encode(text).0;
            self.write_i32(bytes.len() as i32);
            self.write_vec(&bytes)
        } else {
            self.write_i32(len as i32);
            self.write_vec(text.as_bytes());
        };
    }

    pub(crate) fn write_vertex_index(&mut self, size: u8, value: i32) {
        match size {
            1 => self.write_u8(value as u8),
            2 => self.write_u16(value as u16),
            4 => self.write_i32(value),
            _ => {}
        }
    }

    pub(crate) fn write_sized(&mut self, size: u8, value: i32) {
        match size {
            1 => {
                self.write_i8(value as i8);
            }
            2 => {
                self.write_i16(value as i16);
            }
            4 => {
                self.write_i32(value);
            }
            _ => {}
        }
    }

    pub(crate) fn write_face(&mut self, s_vertex_index: u8, face: PMXFace) {
        self.write_vertex_index(s_vertex_index, face.vertices[0]);
        self.write_vertex_index(s_vertex_index, face.vertices[1]);
        self.write_vertex_index(s_vertex_index, face.vertices[2]);
    }

    pub(crate) fn write_pmx_material(&mut self, s_texture_index: u8, material: PMXMaterial) {
        self.write_text_buf(&material.name);
        self.write_text_buf(&material.english_name);
        self.write_vec4(material.diffuse);
        self.write_vec3(material.specular);
        self.write_f32(material.specular_factor);
        self.write_vec3(material.ambient);
        self.write_u8(material.draw_mode);
        self.write_vec4(material.edge_color);
        self.write_f32(material.edge_size);
        self.write_sized(s_texture_index, material.texture_index);
        self.write_sized(s_texture_index, material.sphere_mode_texture_index);

        let spmode = match material.sphere_mode {
            PMXSphereModeRaw::None => 0u8,
            PMXSphereModeRaw::Mul => 1u8,
            PMXSphereModeRaw::Add => 2u8,
            PMXSphereModeRaw::SubTexture => 3u8,
        };
        self.write_u8(spmode);
        let toonmode = match material.toon_mode {
            PMXToonModeRaw::Separate => 0u8,
            PMXToonModeRaw::Common => 1u8,
        };
        self.write_u8(toonmode);
        match material.toon_mode {
            PMXToonModeRaw::Separate => {
                self.write_sized(s_texture_index, material.toon_texture_index)
            }
            PMXToonModeRaw::Common => self.write_sized(1, material.toon_texture_index),
        }
        self.write_text_buf(&material.memo);
        self.write_i32(material.num_face_vertices)
    }

    pub(crate) fn write_pmx_vertex(
        &mut self,
        additional_uvs: u8,
        vertex: PMXVertex,
        s_bone_index: u8,
    ) {
        self.write_vec3(vertex.position);
        self.write_vec3(vertex.norm);
        self.write_vec2(vertex.uv);
        if additional_uvs > 0 {
            for i in 0..additional_uvs {
                self.write_vec4(vertex.add_uv[i as usize]);
            }
        }

        let weight_type = match vertex.weight_type {
            PMXVertexWeight::BDEF1(_) => 0,
            PMXVertexWeight::BDEF2 { .. } => 1,
            PMXVertexWeight::BDEF4 { .. } => 2,
            PMXVertexWeight::SDEF { .. } => 3,
            PMXVertexWeight::QDEF { .. } => 4,
        };
        self.write_u8(weight_type);
        match vertex.weight_type {
            PMXVertexWeight::BDEF1(index) => {
                self.write_sized(s_bone_index, index);
            }
            PMXVertexWeight::BDEF2 {
                bone_index_1,
                bone_index_2,
                bone_weight_1,
            } => {
                self.write_sized(s_bone_index, bone_index_1);
                self.write_sized(s_bone_index, bone_index_2);
                self.write_f32(bone_weight_1);
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
                self.write_sized(s_bone_index, bone_index_1);
                self.write_sized(s_bone_index, bone_index_2);
                self.write_sized(s_bone_index, bone_index_3);
                self.write_sized(s_bone_index, bone_index_4);
                self.write_f32(bone_weight_1);
                self.write_f32(bone_weight_2);
                self.write_f32(bone_weight_3);
                self.write_f32(bone_weight_4);
            }
            PMXVertexWeight::SDEF {
                bone_index_1,
                bone_index_2,
                bone_weight_1,
                sdef_c,
                sdef_r0,
                sdef_r1,
            } => {
                self.write_sized(s_bone_index, bone_index_1);
                self.write_sized(s_bone_index, bone_index_2);
                self.write_f32(bone_weight_1);
                self.write_vec3(sdef_c);
                self.write_vec3(sdef_r0);
                self.write_vec3(sdef_r1);
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
                self.write_sized(s_bone_index, bone_index_1);
                self.write_sized(s_bone_index, bone_index_2);
                self.write_sized(s_bone_index, bone_index_3);
                self.write_sized(s_bone_index, bone_index_4);
                self.write_f32(bone_weight_1);
                self.write_f32(bone_weight_2);
                self.write_f32(bone_weight_3);
                self.write_f32(bone_weight_4);
            }
        }

        self.write_f32(vertex.edge_mag);
    }

    pub(crate) fn write_ik_link(&mut self, s_bone_index: u8, ik_link: PMXIKLink) {
        self.write_sized(s_bone_index, ik_link.ik_bone_index);
        self.write_u8(ik_link.enable_limit);
        if ik_link.enable_limit == 1 {
            self.write_vec3(ik_link.limit_min);
            self.write_vec3(ik_link.limit_max);
        }
    }

    pub(crate) fn write_bone(&mut self, s_bone_index: u8, bone: PMXBone) {
        self.write_text_buf(&bone.name);
        self.write_text_buf(&bone.english_name);
        self.write_vec3(bone.position);
        self.write_sized(s_bone_index, bone.parent);
        self.write_i32(bone.deform_depth);
        self.write_u16(bone.boneflag);
        if (bone.boneflag & BONE_FLAG_TARGET_SHOW_MODE_MASK) == BONE_FLAG_TARGET_SHOW_MODE_MASK {
            self.write_sized(s_bone_index, bone.child);
        } else {
            self.write_vec3(bone.offset);
        }
        if bone.boneflag & (BONE_FLAG_APPEND_ROTATE_MASK | BONE_FLAG_APPEND_TRANSLATE_MASK) > 0 {
            self.write_sized(s_bone_index, bone.append_bone_index);
            self.write_f32(bone.append_weight);
        }
        if (bone.boneflag & BONE_FLAG_FIXED_AXIS_MASK) == BONE_FLAG_FIXED_AXIS_MASK {
            self.write_vec3(bone.fixed_axis);
        }
        if (bone.boneflag & BONE_FLAG_LOCAL_AXIS_MASK) == BONE_FLAG_LOCAL_AXIS_MASK {
            self.write_vec3(bone.local_axis_x);
            self.write_vec3(bone.local_axis_z);
        }
        if (bone.boneflag & BONE_FLAG_DEFORM_OUTER_PARENT_MASK) > 0 {
            self.write_i32(bone.key_value);
        }
        if (bone.boneflag & BONE_FLAG_IK_MASK) == BONE_FLAG_IK_MASK {
            self.write_sized(s_bone_index, bone.ik_target_index);
            self.write_i32(bone.ik_iter_count);
            self.write_f32(bone.ik_limit);
            self.write_i32(bone.ik_links.len() as i32);
            for ik_link in bone.ik_links {
                self.write_ik_link(s_bone_index, ik_link);
            }
        }
    }

    pub(crate) fn write_pmx_morph(
        &mut self,
        s_vertex_index: u8,
        s_bone_index: u8,
        s_material_index: u8,
        s_morph_index: u8,
        s_rigid_index: u8,
        morph: PMXMorph,
    ) {
        self.write_text_buf(&morph.name);
        self.write_text_buf(&morph.english_name);
        self.write_u8(morph.category);
        self.write_u8(morph.morph_type);
        self.write_i32(morph.morph_data.len() as i32);
        for morph_ in morph.morph_data {
            match morph_ {
                MorphTypes::Vertex(morph) => self.write_vertex_morph(s_vertex_index, morph),
                MorphTypes::UV(morph) => self.write_uv_morph(s_vertex_index, morph),
                MorphTypes::UV1(morph) => self.write_uv_morph(s_vertex_index, morph),
                MorphTypes::UV2(morph) => self.write_uv_morph(s_vertex_index, morph),
                MorphTypes::UV3(morph) => self.write_uv_morph(s_vertex_index, morph),
                MorphTypes::UV4(morph) => self.write_uv_morph(s_vertex_index, morph),
                MorphTypes::Bone(morph) => self.write_bone_morph(s_bone_index, morph),
                MorphTypes::Material(morph) => self.write_material_morph(s_material_index, morph),
                MorphTypes::Group(morph) => self.write_group_morph(s_morph_index, morph),
                MorphTypes::Flip(morph) => self.write_flip_morph(s_morph_index, morph),
                MorphTypes::Impulse(morph) => self.write_impulse_morph(s_rigid_index, morph),
            }
        }
    }

    fn write_vertex_morph(&mut self, s_vertex_index: u8, morph: VertexMorph) {
        self.write_sized(s_vertex_index, morph.index);
        self.write_vec3(morph.offset);
    }

    fn write_uv_morph(&mut self, s_vertex_index: u8, morph: UVMorph) {
        self.write_sized(s_vertex_index, morph.index);
        self.write_vec4(morph.offset);
    }

    fn write_bone_morph(&mut self, s_bone_index: u8, morph: BoneMorph) {
        self.write_sized(s_bone_index, morph.index);
        self.write_vec3(morph.translates);
        self.write_vec4(morph.rotates);
    }

    fn write_material_morph(&mut self, s_material_index: u8, morph: MaterialMorph) {
        self.write_sized(s_material_index, morph.index);
        self.write_u8(morph.formula);
        self.write_vec4(morph.diffuse);
        self.write_vec3(morph.specular);
        self.write_f32(morph.specular_factor);
        self.write_vec3(morph.ambient);
        self.write_vec4(morph.edge_color);
        self.write_f32(morph.edge_size);
        self.write_vec4(morph.texture_factor);
        self.write_vec4(morph.sphere_texture_factor);
        self.write_vec4(morph.toon_texture_factor);
    }

    fn write_group_morph(&mut self, s_morph_index: u8, morph: GroupMorph) {
        self.write_sized(s_morph_index, morph.index);
        self.write_f32(morph.morph_factor);
    }

    fn write_flip_morph(&mut self, s_morph_index: u8, morph: FlipMorph) {
        self.write_sized(s_morph_index, morph.index);
        self.write_f32(morph.morph_factor);
    }

    fn write_impulse_morph(&mut self, s_rigid_index: u8, morph: ImpulseMorph) {
        self.write_sized(s_rigid_index, morph.rigid_index);
        self.write_u8(morph.is_local);
        self.write_vec3(morph.velocity);
        self.write_vec3(morph.torque);
    }

    pub(crate) fn write_pmx_frame(&mut self, s_bone_index: u8, s_morph_index: u8, frame: PMXFrame) {
        self.write_text_buf(&frame.name);
        self.write_text_buf(&frame.name_en);
        self.write_u8(frame.is_special);
        self.write_i32(frame.inners.len() as i32);
        for inner in frame.inners {
            self.write_u8(inner.target);
            if inner.target > 1 {
                panic!("A invalid pmx frame detected");
            }
            if inner.target == 0 {
                self.write_sized(s_bone_index, inner.index)
            } else {
                self.write_sized(s_morph_index, inner.index)
            }
        }
    }

    pub(crate) fn write_pmx_rigid(&mut self, s_bone_index: u8, rigid: PMXRigid) {
        self.write_text_buf(&rigid.name);
        self.write_text_buf(&rigid.name_en);
        self.write_sized(s_bone_index, rigid.bone_index);
        self.write_u8(rigid.group);
        self.write_u16(rigid.un_collision_group_flag);
        let form = match rigid.form {
            PMXRigidForm::Sphere => 0,
            PMXRigidForm::Box => 1,
            PMXRigidForm::Capsule => 2,
        };
        self.write_u8(form);

        self.write_vec3(rigid.size);
        self.write_vec3(rigid.position);
        self.write_vec3(rigid.rotation);
        self.write_f32(rigid.mass);
        self.write_f32(rigid.move_resist);
        self.write_f32(rigid.rotation_resist);
        self.write_f32(rigid.repulsion);
        self.write_f32(rigid.friction);
        let calc_method = match rigid.calc_method {
            PMXRigidCalcMethod::Static => 0,
            PMXRigidCalcMethod::Dynamic => 1,
            PMXRigidCalcMethod::DynamicWithBonePosition => 2,
        };
        self.write_u8(calc_method);
    }

    pub(crate) fn write_pmx_joint(&mut self, s_rigid_index: u8, joint: PMXJoint) {
        self.write_text_buf(&joint.name);
        self.write_text_buf(&joint.name_en);
        let kind = match joint.joint_type {
            PMXJointType::Spring6DOF { .. } => 0,
            PMXJointType::_6DOF { .. } => 1,
            PMXJointType::P2P { .. } => 2,
            PMXJointType::ConeTwist { .. } => 3,
            PMXJointType::Slider { .. } => 4,
            PMXJointType::Hinge { .. } => 5,
        };
        self.write_u8(kind);
        match joint.joint_type {
            PMXJointType::Spring6DOF {
                a_rigid_index,
                b_rigid_index,
                position,
                rotation,
                move_limit_down,
                move_limit_up,
                rotation_limit_down,
                rotation_limit_up,
                spring_const_move,
                spring_const_rotation,
            } => {
                self.write_sized(s_rigid_index, a_rigid_index);
                self.write_sized(s_rigid_index, b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3(spring_const_move);
                self.write_vec3(spring_const_rotation);
            }
            PMXJointType::_6DOF {
                a_rigid_index,
                b_rigid_index,
                position,
                rotation,
                move_limit_down,
                move_limit_up,
                rotation_limit_down,
                rotation_limit_up,
            } => {
                self.write_sized(s_rigid_index, a_rigid_index);
                self.write_sized(s_rigid_index, b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3([0.0, 0.0, 0.0]);
            }
            PMXJointType::P2P {
                a_rigid_index,
                b_rigid_index,
                position,
                rotation,
            } => {
                let dummy = [0.0, 0.0, 0.0];
                self.write_sized(s_rigid_index, a_rigid_index);
                self.write_sized(s_rigid_index, b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
            }
            PMXJointType::ConeTwist {
                a_rigid_index,
                b_rigid_index,
                swing_span1,
                swing_span2,
                twist_span,
                softness,
                bias_factor,
                relaxation_factor,
                damping,
                fix_thresh,
                enable_motor,
                max_motor_impulse,
                motor_target_in_constraint_space,
            } => {
                let dummy = [0.0, 0.0, 0.0];
                let position = dummy;
                let rotation = dummy;
                let move_limit_down = [damping, 0.0, if enable_motor { 1.0 } else { 0.0 }];
                let move_limit_up = [fix_thresh, 0.0, max_motor_impulse];
                let rotation_limit_down = [twist_span, swing_span2, swing_span1];
                let rotation_limit_up = dummy;
                let spring_const_move = [softness, bias_factor, relaxation_factor];
                let spring_const_rotation = motor_target_in_constraint_space;
                self.write_sized(s_rigid_index, a_rigid_index);
                self.write_sized(s_rigid_index, b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3(spring_const_move);
                self.write_vec3(spring_const_rotation);
            }
            PMXJointType::Slider {
                a_rigid_index,
                b_rigid_index,
                lower_linear_limit,
                upper_linear_limit,
                lower_angle_limit,
                upper_angle_limit,
                power_linear_motor,
                target_linear_motor_velocity,
                max_linear_motor_force,
                power_angler_motor,
                target_angler_motor_velocity,
                max_angler_motor_force,
            } => {
                let move_limit_down = [lower_linear_limit, 0.0, 0.0];
                let move_limit_up = [upper_linear_limit, 0.0, 0.0];
                let rotation_limit_down = [lower_angle_limit, 0.0, 0.0];
                let rotation_limit_up = [upper_angle_limit, 0.0, 0.0];
                let spring_const_move = [
                    if power_linear_motor { 1.0 } else { 0.0 },
                    target_linear_motor_velocity,
                    max_linear_motor_force,
                ];
                let spring_const_rotation = [
                    if power_angler_motor { 1.0 } else { 0.0 },
                    target_angler_motor_velocity,
                    max_angler_motor_force,
                ];
                self.write_sized(s_rigid_index, a_rigid_index);
                self.write_sized(s_rigid_index, b_rigid_index);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3(spring_const_move);
                self.write_vec3(spring_const_rotation);
            }
            PMXJointType::Hinge {
                a_rigid_index,
                b_rigid_index,
                low,
                high,
                softness,
                bias_factor,
                relaxation_factor,
                enable_motor,
                target_velocity,
                max_motor_impulse,
            } => {
                let rotation_limit_down = [low, 0.0, 0.0];
                let rotation_limit_up = [high, 0.0, 0.0];
                let spring_const_move = [softness, bias_factor, relaxation_factor];
                let spring_const_rotation = [
                    if enable_motor { 1.0 } else { 0.0 },
                    target_velocity,
                    max_motor_impulse,
                ];
                self.write_sized(s_rigid_index, a_rigid_index);
                self.write_sized(s_rigid_index, b_rigid_index);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3(spring_const_move);
                self.write_vec3(spring_const_rotation);
            }
        }
    }

    write_bin!(write_vec4, Vec4);
    write_bin!(write_vec3, Vec3);
    write_bin!(write_vec2, Vec2);
    write_bin!(write_f32, f32);
    write_bin!(write_i32, i32);
    write_bin!(write_u32, u32);
    write_bin!(write_i16, i16);
    write_bin!(write_u16, u16);
    write_bin!(write_i8, i8);
    write_bin!(write_u8, u8);
}
