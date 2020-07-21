macro_rules! read_bin {
    ($F:ident,$T:ident) => {
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

    use crate::pmx_loader::pmx_loader::*;
    use crate::pmx_loader::{MaterialsLoader, TexturesLoader};

    #[test]
    fn it_works() {
        let filename = env::args().skip(1).next().unwrap();
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
}
