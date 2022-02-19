use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::Write;
use std::path::Path;

use crate::types::{
    Bone, BoneMorph, ConnectionDisplayMode, ControlPanel, Encode, Face, FlipMorph, Frame,
    FrameInner, GroupMorph, Header, IKLink, ImpulseMorph, IndexKinds, Joint, JointType, Material,
    MaterialMorph, Morph, MorphKinds, PMXVersion, Rigid, RigidCalcMethod, RigidForm,
    RotateAndTranslateInherits, SoftBody, SoftBodyAeroModel, SoftBodyForm, SphereModeKind,
    ToonMode, UVMorph, Vertex, VertexIndexKinds, VertexMorph, VertexWeight,
};
use crate::types::{Vec2, Vec3, Vec4};
use std::convert::TryFrom;

/// This is internal use only struct
/// Do not use this struct
pub(crate) struct BinaryWriter<W: Write> {
    pub(crate) inner: BufWriter<W>,
    pub(crate) header: Header,
}

macro_rules! write_bin {
    ($F:ident,$T:ty) => {
        ///Macro implemented member for internal use
        pub(crate) fn $F(&mut self, value: $T) {
            let buf = value.to_le_bytes();
            self.inner.write_all(&buf).unwrap();
        }
    };
}
macro_rules! write_bin_array {
    ($F:ident,$T:ty) => {
        ///Macro implemented member for internal use
        pub(crate) fn $F(&mut self, value: $T) {
            for element in value {
                let buf = element.to_le_bytes();
                self.inner.write_all(&buf).unwrap();
            }
        }
    };
}
impl BinaryWriter<File> {
    pub(crate) fn create<P: AsRef<Path>>(path: P, header: Header) -> Result<Self, Error> {
        let file = File::create(&path);

        match file {
            Ok(file) => {
                let inner = BufWriter::with_capacity(1024, file);
                Ok(BinaryWriter { inner, header })
            }
            Err(err) => Err(err),
        }
    }
}
impl<W: Write> BinaryWriter<W> {
    pub(crate) fn from_writer(writer: W, header: Header) -> Self {
        Self {
            inner: BufWriter::new(writer),
            header,
        }
    }
    pub(crate) fn write_header(&mut self) {
        let magic = self.header.magic.clone();
        let bytes = magic.as_bytes();
        self.write_vec(bytes);
        self.write_f32(match self.header.version {
            PMXVersion::V20 => 2.0,
            PMXVersion::V21 => 2.1,
        });
        self.write_u8(self.header.length);
        self.write_u8(match self.header.encode {
            Encode::UTF8 => 1,
            Encode::Utf16Le => 0,
        });
        self.write_u8(self.header.additional_uv);
        self.write_u8(u8::from(self.header.s_vertex_index));
        self.write_u8(u8::from(self.header.s_texture_index));
        self.write_u8(u8::from(self.header.s_material_index));
        self.write_u8(u8::from(self.header.s_bone_index));
        self.write_u8(u8::from(self.header.s_morph_index));
        self.write_u8(u8::from(self.header.s_rigid_body_index));
    }
    pub(crate) fn write_vec(&mut self, v: &[u8]) {
        self.inner.write_all(v).unwrap();
    }

    pub(crate) fn write_text_buf(&mut self, text: &str) {
        let len = text.len();
        if self.header.encode == Encode::Utf16Le {
            let utf_16_sequence: Vec<u16> = text.encode_utf16().map(u16::to_le).collect();
            self.write_i32(i32::try_from(utf_16_sequence.len() * 2).unwrap());
            for ch in utf_16_sequence {
                self.write_u16(ch);
            }
        } else {
            self.write_i32(i32::try_from(len).unwrap());
            self.write_vec(text.as_bytes());
        };
    }

    pub(crate) fn write_vertex_index(&mut self, value: i32) {
        match self.header.s_vertex_index {
            VertexIndexKinds::U8 => self.write_u8(u8::try_from(value).unwrap()),
            VertexIndexKinds::U16 => self.write_u16(u16::try_from(value).unwrap()),
            VertexIndexKinds::I32 => self.write_i32(value),
        }
    }

    fn write_sized(&mut self, size: IndexKinds, value: i32) {
        match size {
            IndexKinds::I8 => {
                self.write_i8(i8::try_from(value).unwrap());
            }
            IndexKinds::I16 => {
                self.write_i16(i16::try_from(value).unwrap());
            }
            IndexKinds::I32 => {
                self.write_i32(value);
            }
        }
    }
    fn write_texture_index(&mut self, value: i32) {
        self.write_sized(self.header.s_texture_index, value);
    }
    fn write_material_index(&mut self, value: i32) {
        self.write_sized(self.header.s_material_index, value);
    }
    fn write_bone_index(&mut self, value: i32) {
        self.write_sized(self.header.s_bone_index, value);
    }
    fn write_morph_index(&mut self, value: i32) {
        self.write_sized(self.header.s_morph_index, value);
    }
    fn write_rigid_index(&mut self, value: i32) {
        self.write_sized(self.header.s_rigid_body_index, value);
    }
    pub(crate) fn write_face(&mut self, face: &Face) {
        self.write_vertex_index(face.vertices[0]);
        self.write_vertex_index(face.vertices[1]);
        self.write_vertex_index(face.vertices[2]);
    }

    pub(crate) fn write_material(&mut self, material: &Material) {
        self.write_text_buf(&material.name);
        self.write_text_buf(&material.english_name);
        self.write_vec4(material.diffuse);
        self.write_vec3(material.specular);
        self.write_f32(material.specular_factor);
        self.write_vec3(material.ambient);
        self.write_u8(material.draw_mode.bits());
        self.write_vec4(material.edge_color);
        self.write_f32(material.edge_size);
        self.write_texture_index(material.texture_index);
        if let Some(sp_mode) = material.sphere_mode {
            self.write_texture_index(sp_mode.index);
            self.write_u8(match sp_mode.kind {
                SphereModeKind::Mul => 1,
                SphereModeKind::Add => 2,
                SphereModeKind::SubTexture => 3,
            });
        } else {
            self.write_texture_index(-1);
            self.write_u8(0);
        }
        match material.toon_mode {
            ToonMode::Separate(idx) => {
                self.write_u8(0);
                self.write_texture_index(idx);
            }
            ToonMode::Common(idx) => {
                self.write_u8(1);
                self.write_u8(idx);
            }
        }

        self.write_text_buf(&material.memo);
        self.write_i32(material.num_face_vertices);
    }

    pub(crate) fn write_vertex(&mut self, vertex: &Vertex) {
        self.write_vec3(vertex.position);
        self.write_vec3(vertex.norm);
        self.write_vec2(vertex.uv);

        for i in 0..self.header.additional_uv {
            self.write_vec4(vertex.add_uv[i as usize]);
        }

        let weight_type = match vertex.weight_type {
            VertexWeight::BDEF1(_) => 0,
            VertexWeight::BDEF2 { .. } => 1,
            VertexWeight::BDEF4 { .. } => 2,
            VertexWeight::SDEF { .. } => 3,
            VertexWeight::QDEF { .. } => 4,
        };
        self.write_u8(weight_type);
        match vertex.weight_type {
            VertexWeight::BDEF1(index) => {
                self.write_bone_index(index);
            }
            VertexWeight::BDEF2 {
                bone_index_1,
                bone_index_2,
                bone_weight_1,
            } => {
                self.write_bone_index(bone_index_1);
                self.write_bone_index(bone_index_2);
                self.write_f32(bone_weight_1);
            }
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
            | VertexWeight::QDEF {
                bone_index_1,
                bone_index_2,
                bone_index_3,
                bone_index_4,
                bone_weight_1,
                bone_weight_2,
                bone_weight_3,
                bone_weight_4,
            } => {
                self.write_bone_index(bone_index_1);
                self.write_bone_index(bone_index_2);
                self.write_bone_index(bone_index_3);
                self.write_bone_index(bone_index_4);
                self.write_f32(bone_weight_1);
                self.write_f32(bone_weight_2);
                self.write_f32(bone_weight_3);
                self.write_f32(bone_weight_4);
            }
            VertexWeight::SDEF {
                bone_index_1,
                bone_index_2,
                bone_weight_1,
                sdef_c,
                sdef_r0,
                sdef_r1,
            } => {
                self.write_bone_index(bone_index_1);
                self.write_bone_index(bone_index_2);
                self.write_f32(bone_weight_1);
                self.write_vec3(sdef_c);
                self.write_vec3(sdef_r0);
                self.write_vec3(sdef_r1);
            }
        }

        self.write_f32(vertex.edge_mag);
    }

    pub(crate) fn write_ik_link(&mut self, ik_link: &IKLink) {
        self.write_bone_index(ik_link.ik_bone_index);
        self.write_bool(ik_link.angle_limit.is_some());
        if let Some(limits) = ik_link.angle_limit {
            self.write_vec3(limits.0);
            self.write_vec3(limits.1);
        }
    }

    pub(crate) fn write_bone(&mut self, bone: &Bone) {
        self.write_text_buf(&bone.name);
        self.write_text_buf(&bone.english_name);
        self.write_vec3(bone.position);
        self.write_bone_index(bone.parent);
        self.write_i32(bone.deform_depth);
        let bone_flags = bone.calculate_bone_flag();
        self.write_u16(bone_flags.bits());
        match bone.connection_display_mode {
            ConnectionDisplayMode::OtherBone(x) => {
                self.write_bone_index(x);
            }
            ConnectionDisplayMode::Offset(x) => {
                self.write_vec3(x);
            }
        }
        match bone.inherits.rotate_and_translate {
            RotateAndTranslateInherits::None => {}
            RotateAndTranslateInherits::Both(bone_index, y)
            | RotateAndTranslateInherits::Rotate(bone_index, y)
            | RotateAndTranslateInherits::Translate(bone_index, y) => {
                self.write_bone_index(bone_index);
                self.write_f32(y);
            }
        }
        if let Some(axis) = bone.fixed_axis {
            self.write_vec3(axis);
        }
        if let Some((local_axis_x, local_axis_z)) = bone.local_axis {
            self.write_vec3(local_axis_x);
            self.write_vec3(local_axis_z);
        }
        if let Some(key) = bone.external_parent {
            self.write_i32(key);
        }
        if let Some(ik_infos) = &bone.ik_info {
            self.write_bone_index(ik_infos.ik_target_bone_index);
            self.write_i32(ik_infos.ik_iter_count);
            self.write_f32(ik_infos.ik_limit_angle);
            self.write_i32(i32::try_from(ik_infos.ik_links.len()).unwrap());
            ik_infos
                .ik_links
                .iter()
                .for_each(|ik_link| self.write_ik_link(ik_link));
        }
    }

    pub(crate) fn write_morph(&mut self, morph: &Morph) {
        self.write_text_buf(&morph.name);
        self.write_text_buf(&morph.english_name);
        self.write_u8(match morph.control_panel {
            ControlPanel::BottomLeft => 1,
            ControlPanel::TopLeft => 2,
            ControlPanel::TopRight => 3,
            ControlPanel::BottomRight => 4,
            ControlPanel::System => 0,
        });
        match &morph.morph_data {
            MorphKinds::Group(x) => {
                self.write_u8(0);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_group_morph(*x));
            }
            MorphKinds::Vertex(x) => {
                self.write_u8(1);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_vertex_morph(x));
            }
            MorphKinds::Bone(x) => {
                self.write_u8(2);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_bone_morph(x));
            }
            MorphKinds::UV(x) => {
                self.write_u8(3);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_uv_morph(x));
            }
            MorphKinds::UV1(x) => {
                self.write_u8(4);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_uv_morph(x));
            }
            MorphKinds::UV2(x) => {
                self.write_u8(5);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_uv_morph(x));
            }
            MorphKinds::UV3(x) => {
                self.write_u8(6);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_uv_morph(x));
            }
            MorphKinds::UV4(x) => {
                self.write_u8(7);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_uv_morph(x));
            }
            MorphKinds::Material(x) => {
                self.write_u8(8);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_material_morph(x));
            }
            MorphKinds::Flip(x) => {
                self.write_u8(9);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_flip_morph(x));
            }
            MorphKinds::Impulse(x) => {
                self.write_u8(10);
                self.write_i32(i32::try_from(x.len()).unwrap());
                x.iter().for_each(|x| self.write_impulse_morph(x));
            }
        }
    }

    fn write_vertex_morph(&mut self, morph: &VertexMorph) {
        self.write_vertex_index(morph.index);
        self.write_vec3(morph.offset);
    }

    fn write_uv_morph(&mut self, morph: &UVMorph) {
        self.write_vertex_index(morph.index);
        self.write_vec4(morph.offset);
    }

    fn write_bone_morph(&mut self, morph: &BoneMorph) {
        self.write_bone_index(morph.index);
        self.write_vec3(morph.translates);
        self.write_vec4(morph.rotates);
    }

    fn write_material_morph(&mut self, morph: &MaterialMorph) {
        self.write_material_index(morph.index);
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

    fn write_group_morph(&mut self, morph: GroupMorph) {
        self.write_morph_index(morph.index);
        self.write_f32(morph.morph_factor);
    }

    fn write_flip_morph(&mut self, morph: &FlipMorph) {
        self.write_morph_index(morph.index);
        self.write_f32(morph.morph_factor);
    }

    fn write_impulse_morph(&mut self, morph: &ImpulseMorph) {
        self.write_rigid_index(morph.rigid_index);
        self.write_bool(morph.is_local);
        self.write_vec3(morph.velocity);
        self.write_vec3(morph.torque);
    }

    pub(crate) fn write_frame(&mut self, frame: &Frame) {
        self.write_text_buf(&frame.name);
        self.write_text_buf(&frame.name_en);
        self.write_bool(frame.is_special);
        self.write_i32(i32::try_from(frame.inners.len()).unwrap());
        frame.inners.iter().for_each(|inner| match inner {
            FrameInner::Bone(index) => {
                self.write_u8(0);
                self.write_bone_index(*index);
            }
            FrameInner::Morph(index) => {
                self.write_u8(1);
                self.write_morph_index(*index);
            }
        });
    }

    pub(crate) fn write_rigid(&mut self, rigid: &Rigid) {
        self.write_text_buf(&rigid.name);
        self.write_text_buf(&rigid.name_en);
        self.write_bone_index(rigid.bone_index);
        self.write_u8(rigid.group);
        self.write_u16(rigid.un_collision_group_flag);
        let form = match rigid.form {
            RigidForm::Sphere => 0,
            RigidForm::Box => 1,
            RigidForm::Capsule => 2,
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
            RigidCalcMethod::Static => 0,
            RigidCalcMethod::Dynamic => 1,
            RigidCalcMethod::DynamicWithBonePosition => 2,
        };
        self.write_u8(calc_method);
    }

    pub(crate) fn write_joint(&mut self, joint: &Joint) {
        self.write_text_buf(&joint.name);
        self.write_text_buf(&joint.name_en);
        let kind = match joint.joint_type {
            JointType::Spring6DOF { .. } => 0,
            JointType::SixDof { .. } => 1,
            JointType::P2P { .. } => 2,
            JointType::ConeTwist { .. } => 3,
            JointType::Slider { .. } => 4,
            JointType::Hinge { .. } => 5,
        };
        self.write_u8(kind);
        match joint.joint_type {
            JointType::Spring6DOF {
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
                self.write_rigid_index(a_rigid_index);
                self.write_rigid_index(b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3(spring_const_move);
                self.write_vec3(spring_const_rotation);
            }
            JointType::SixDof {
                a_rigid_index,
                b_rigid_index,
                position,
                rotation,
                move_limit_down,
                move_limit_up,
                rotation_limit_down,
                rotation_limit_up,
            } => {
                self.write_rigid_index(a_rigid_index);
                self.write_rigid_index(b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3([0.0, 0.0, 0.0]);
            }
            JointType::P2P {
                a_rigid_index,
                b_rigid_index,
                position,
                rotation,
            } => {
                let dummy = [0.0, 0.0, 0.0];
                self.write_rigid_index(a_rigid_index);
                self.write_rigid_index(b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
                self.write_vec3(dummy);
            }
            JointType::ConeTwist {
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
                self.write_rigid_index(a_rigid_index);
                self.write_rigid_index(b_rigid_index);
                self.write_vec3(position);
                self.write_vec3(rotation);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3(spring_const_move);
                self.write_vec3(spring_const_rotation);
            }
            JointType::Slider {
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
                self.write_rigid_index(a_rigid_index);
                self.write_rigid_index(b_rigid_index);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3([0.0, 0.0, 0.0]);
                self.write_vec3(move_limit_down);
                self.write_vec3(move_limit_up);
                self.write_vec3(rotation_limit_down);
                self.write_vec3(rotation_limit_up);
                self.write_vec3(spring_const_move);
                self.write_vec3(spring_const_rotation);
            }
            JointType::Hinge {
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
                self.write_rigid_index(a_rigid_index);
                self.write_rigid_index(b_rigid_index);
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

    pub fn write_soft_body(&mut self, soft_body: &SoftBody) {
        self.write_text_buf(&soft_body.name);
        self.write_text_buf(&soft_body.name_en);
        match soft_body.form {
            SoftBodyForm::TriMesh => self.write_u8(0),
            SoftBodyForm::Rope => self.write_u8(1),
        }
        self.write_material_index(soft_body.material_index);
        self.write_u8(soft_body.group);
        self.write_u16(soft_body.un_collision_group_flag);
        self.write_u8(soft_body.bit_flag);
        self.write_i32(soft_body.b_link_create_distance);
        self.write_i32(soft_body.clusters);
        self.write_f32(soft_body.mass);
        self.write_f32(soft_body.collision_margin);
        let aero_model = match soft_body.aero_model {
            SoftBodyAeroModel::VPoint => 0,
            SoftBodyAeroModel::VTwoSide => 1,
            SoftBodyAeroModel::VOneSided => 2,
            SoftBodyAeroModel::FTwoSided => 3,
            SoftBodyAeroModel::FOneSided => 4,
        };
        self.write_i32(aero_model);
        //config
        self.write_f32(soft_body.vcf);
        self.write_f32(soft_body.dp);
        self.write_f32(soft_body.dg);
        self.write_f32(soft_body.lf);
        self.write_f32(soft_body.pr);
        self.write_f32(soft_body.vc);
        self.write_f32(soft_body.df);
        self.write_f32(soft_body.mt);
        self.write_f32(soft_body.chr);
        self.write_f32(soft_body.khr);
        self.write_f32(soft_body.shr);
        self.write_f32(soft_body.ahr);
        //clusters
        self.write_f32(soft_body.srhr_cl);
        self.write_f32(soft_body.skhr_cl);
        self.write_f32(soft_body.sshr_cl);
        self.write_f32(soft_body.sr_splt_cl);
        self.write_f32(soft_body.sk_splt_cl);
        self.write_f32(soft_body.ss_splt_cl);
        //iteration
        self.write_i32(soft_body.v_it);
        self.write_i32(soft_body.p_it);
        self.write_i32(soft_body.d_it);
        self.write_i32(soft_body.c_it);
        //material
        self.write_f32(soft_body.lst);
        self.write_f32(soft_body.ast);
        self.write_f32(soft_body.vst);
        //anchor rigid
        self.write_i32(i32::try_from(soft_body.anchor_rigid.len()).unwrap());
        soft_body.anchor_rigid.iter().for_each(|anchor_rigid| {
            self.write_rigid_index(anchor_rigid.rigid_index);
            self.write_vertex_index(anchor_rigid.vertex_index);
            self.write_bool(anchor_rigid.near_mode);
        });

        //pin vertex
        self.write_i32(i32::try_from(soft_body.pin_vertex.len()).unwrap());
        soft_body
            .pin_vertex
            .iter()
            .for_each(|vertex_index| self.write_vertex_index(*vertex_index));
    }
    write_bin_array!(write_vec4, Vec4);
    write_bin_array!(write_vec3, Vec3);
    write_bin_array!(write_vec2, Vec2);
    write_bin!(write_f32, f32);
    write_bin!(write_i32, i32);
    write_bin!(write_i16, i16);
    write_bin!(write_u16, u16);
    write_bin!(write_i8, i8);
    write_bin!(write_u8, u8);
    /// write `true` as `1_u8` , `false` as `0_u8`
    fn write_bool(&mut self, v: bool) {
        if v {
            self.write_u8(1);
        } else {
            self.write_u8(0);
        }
    }
}
