mod pmx_writer;
mod binary_writer;
macro_rules! read_bin {
    ($F:ident,$T:ty) => {
        pub fn $F(&mut self) -> $T {
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
pub mod pmx_loader;
pub mod pmx_types;

#[cfg(test)]
mod test {
    use std::env;


    use crate::pmx_loader::{MaterialsLoader, TexturesLoader, BonesLoader, MorphsLoader, PMXLoader, ModelInfoLoader, VerticesLoader, FacesLoader};
    use crate::pmx_writer::PMXWriter;
    use crate::pmx_types::pmx_types::PMXModelInfo;

    #[test]
    fn loader_test() {

        let filename = "./from.pmx";
        let mut loader = PMXLoader::open(filename);
        let header = loader.get_header();
        println!("{:#?}", header);
        let (model_info, ns) = ModelInfoLoader::read_pmx_model_info(loader);
        print!("{:#?}", model_info);
        let (vertices, ns) = VerticesLoader::read_pmx_vertices(ns);
        print!("{}", vertices);
        let (faces, ns) = FacesLoader::read_pmx_faces(ns);
        println!("{}", faces);
        let (textures, ns) = TexturesLoader::read_texture_list(ns);
        println!("{}", textures);
        let (materials, ns) = MaterialsLoader::read_pmx_materials(ns);
        println!("{:#?}", materials);
    }
    //Perform Copy test
    #[test]
    fn writer_test(){
        let from ="./from.pmx";
        let to ="./to.pmx";
        let mut writer =PMXWriter::begin_writer(to);
        let mut copy_from=PMXLoader::open(from);
        let (model_info,ns)=ModelInfoLoader::read_pmx_model_info(copy_from);
        let (vertices,ns)=VerticesLoader::read_pmx_vertices(ns);
        let (faces,ns)=FacesLoader::read_pmx_faces(ns);
        let(textures,ns)=TexturesLoader::read_texture_list(ns);
        let (materials,ns)=MaterialsLoader::read_pmx_materials(ns);
        let(bones,ns)=BonesLoader::read_pmx_bones(ns);
        let(morphs,ns)=MorphsLoader::read_pmx_morphs(ns);

        writer.set_model_info(Some(&model_info.name),Some(&model_info.name_en),Some(&model_info.comment),Some(&model_info.comment_en));
        writer.add_vertices(&vertices.vertices);
        writer.add_faces(&faces.faces);
        writer.add_textures(&textures.textures);
        writer.add_materials(&materials.materials);
        writer.add_bones(&bones.bones);
        writer.add_morphs(&morphs.morphs);
        PMXWriter::write(writer);
        let reader=PMXLoader::open(to);
        let (model_info_cpy,ns)=ModelInfoLoader::read_pmx_model_info(reader);
        let (vertices_cpy,ns)=VerticesLoader::read_pmx_vertices(ns);
        let (faces_cpy,ns)=FacesLoader::read_pmx_faces(ns);
        let(textures_cpy,ns)=TexturesLoader::read_texture_list(ns);

        let (materials_cpy,ns)=MaterialsLoader::read_pmx_materials(ns);

        let(bones_cpy,ns)=BonesLoader::read_pmx_bones(ns);
        /*
        let(morphs_cpy,ns)=MorphsLoader::read_pmx_morphs(ns);
        */

        assert_eq!(model_info,model_info_cpy);
        assert_eq!(vertices,vertices_cpy);
        assert_eq!(faces,faces_cpy);
        assert_eq!(textures,textures_cpy);
        assert_eq!(materials,materials_cpy);
        assert_eq!(bones,bones_cpy);
    }
}
