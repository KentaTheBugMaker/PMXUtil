//! `pmx_util` - PMX reading and writing utility
//!
//! This crate support PMX 2.0/2.1. but there are very few models use 2.1.
//!
//! ## please note.
//!
//! ``
//! in future release may drop PMX 2.1 support
//! ``
//!
//!  ## PMX 2.0 read and write tested by following steps.
//!
//! * read model from file.
//! * write it to another file
//! * read from wrote file.
//! * compare both content
//! * compare hash.
//! * load it by `PMXEditor` and `MMD`.

pub(crate) mod binary_writer;
pub mod writer;

pub(crate) mod binary_reader;

pub mod reader;
pub mod types;

#[cfg(test)]
mod test {

    use crate::reader::ModelInfoStage;

    use crate::writer::Writer;

    //Perform Copy test
    #[test]
    fn copy_test() {
        let path = std::env::var("PMX_FILE").unwrap();
        let to = "./to.pmx";
        let mut writer = Writer::begin_writer(true);
        let copy_from = crate::reader::ModelInfoStage::open(path).unwrap();
        let (model_info, ns) = copy_from.read();
        let (vertices, ns) = ns.read();
        let (faces, ns) = ns.read();
        let (textures, ns) = ns.read();
        let (materials, ns) = ns.read();
        let (bones, ns) = ns.read();
        let (morphs, ns) = ns.read();
        let (frames, ns) = ns.read();
        let (rigid_bodies, ns) = ns.read();
        let (joints, _ns) = ns.read();

        writer.set_model_info(&model_info);
        writer.add_vertices(&vertices);
        writer.add_faces(&faces);
        writer.add_textures(&textures);
        writer.add_materials(&materials);
        writer.add_bones(&bones);
        writer.add_morphs(&morphs);
        writer.add_frames(&frames);
        writer.add_rigid_bodies(&rigid_bodies);
        writer.add_joints(&joints);
        writer.write_to_path(to).unwrap();

        let reader = ModelInfoStage::open(to).unwrap();
        let (model_info_cpy, ns) = reader.read();
        assert_eq!(model_info, model_info_cpy);
        let (vertices_cpy, ns) = ns.read();
        assert_eq!(vertices, vertices_cpy);
        let (faces_cpy, ns) = ns.read();
        assert_eq!(faces, faces_cpy);
        let (textures_cpy, ns) = ns.read();
        assert_eq!(textures, textures_cpy);
        let (materials_cpy, ns) = ns.read();
        assert_eq!(materials, materials_cpy);
        let (bones_cpy, ns) = ns.read();
        assert_eq!(bones, bones_cpy);
        let (morphs_cpy, ns) = ns.read();
        assert_eq!(morphs, morphs_cpy);
        let (frames_cpy, ns) = ns.read();
        assert_eq!(frames, frames_cpy);
        let (rigid_bodies_cpy, ns) = ns.read();
        assert_eq!(rigid_bodies, rigid_bodies_cpy);
        let (joints_cpy, _ns) = ns.read();
        assert_eq!(joints, joints_cpy);
    }
}
