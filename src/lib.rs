pub mod binary_writer;
pub mod pmx_writer;
macro_rules! read_bin {
    ($F:ident,$T:ty) => {
        pub(crate) fn $F(&mut self) -> $T {
            let temp;
            let mut buf = [0u8; std::mem::size_of::<$T>()];
            self.inner.read_exact(&mut buf).unwrap();
            unsafe {
                temp = transmute(buf);
            }
            temp
        }
    };
}

pub mod binary_reader;
/// PMX loader
///
/// ```rust
/// extern crate PMXUtil;
/// use PMXUtil::pmx_loader;
/// let modelinfo_loader=pmx_loader::PMXLoader::open("/path/to/pmxfile");
/// let (modelinfo,vertices_loader)=modelinfo_loader.read_pmx_model_info();
/// ```
pub mod pmx_loader;
pub mod pmx_types;

#[cfg(test)]
mod test {
    

    use crate::pmx_loader::{
        PMXLoader,
    };
    
    use crate::pmx_writer::PMXWriter;

    //Perform Copy test
    #[test]
    fn copy_test() {
        let from = "./from.pmx";
        let to = "./to.pmx";
        let mut writer = PMXWriter::begin_writer(to);
        let copy_from = PMXLoader::open(from);
        let (model_info, ns) = copy_from.read_pmx_model_info();
        let (vertices, ns) = ns.read_pmx_vertices();
        let (faces, ns) = ns.read_pmx_faces();
        let (textures, ns) = ns.read_texture_list();
        let (materials, ns) = ns.read_pmx_materials();
        let (bones, ns) = ns.read_pmx_bones();
        let (morphs, ns) = ns.read_pmx_morphs();
        let (frames, ns) = ns.read_frames();
        let (rigid_bodies, ns) = ns.read_rigids();
        let (joints, _ns) = ns.read_joints();

        writer.set_model_info(
            Some(&model_info.name),
            Some(&model_info.name_en),
            Some(&model_info.comment),
            Some(&model_info.comment_en),
        );
        writer.add_vertices(&vertices);
        writer.add_faces(&faces);
        writer.add_textures(&textures.textures);
        writer.add_materials(&materials);
        writer.add_bones(&bones);
        writer.add_morphs(&morphs);
        writer.add_frames(&frames);
        writer.add_rigid_bodies(&rigid_bodies);
        writer.add_joints(&joints);
        PMXWriter::write(writer);

        let reader = PMXLoader::open(to);
        let (model_info_cpy, ns) = reader.read_pmx_model_info();
        assert_eq!(model_info, model_info_cpy);
        let (vertices_cpy, ns) = ns.read_pmx_vertices();
        assert_eq!(vertices, vertices_cpy);
        let (faces_cpy, ns) = ns.read_pmx_faces();
        assert_eq!(faces, faces_cpy);
        let (textures_cpy, ns) = ns.read_texture_list();
        assert_eq!(textures, textures_cpy);
        let (materials_cpy, ns) = ns.read_pmx_materials();
        assert_eq!(materials, materials_cpy);
        let (bones_cpy, ns) = ns.read_pmx_bones();
        assert_eq!(bones, bones_cpy);
        let (morphs_cpy, ns) = ns.read_pmx_morphs();
        assert_eq!(morphs, morphs_cpy);
        let (frames_cpy, ns) = ns.read_frames();
        assert_eq!(frames, frames_cpy);
        let (rigid_bodies_cpy, ns) = ns.read_rigids();
        assert_eq!(rigid_bodies, rigid_bodies_cpy);
        let (joints_cpy, _ns) = ns.read_joints();
        assert_eq!(joints, joints_cpy);
    }
}
