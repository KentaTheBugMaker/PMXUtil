macro_rules! read_bin {
    ($F:ident,$T:ident)=>{
          pub fn $F(&mut self)->$T{
            let  temp;
            let mut buf=[0u8;std::mem::size_of::<$T>()];
            self.inner.read_exact(&mut buf).unwrap();
            unsafe{
                temp=transmute(buf);
            }
            temp
            }
    }
}
pub mod binary_reader;
pub mod pmx_loader;
pub mod pmx_types;

#[cfg(test)]
mod test {
    use std::env;

    use crate::pmx_loader::pmx_loader::PMXLoader;

    #[test]
    fn it_works() {
        let filename = env::args().skip(1).next().unwrap();
        let mut loader = PMXLoader::open(filename);
        let header = loader.get_header();
        println!("{:#?}", header);
        let model_info = loader.read_pmx_model_info().unwrap();
        print!("{:#?}", model_info);
        let vertices = loader.read_pmx_vertices().unwrap();
        print!("{}", vertices);
        let faces = loader.read_pmx_faces().unwrap();
        println!("{}", faces);
        let textures = loader.read_texture_list().unwrap();
        println!("{}", textures);
        let materials = loader.read_pmx_materials().unwrap();
        println!("{:#?}", materials);
    }
}