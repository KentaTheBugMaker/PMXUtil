### PMXLoader
## A simple PMX loader written in Rust
### What can this crate do
  1. Parse PMX 2.0/2.1 _header
  2. Parse PMX 2.0/2.1 Model Info
      - Name
      - English Name
      - Comment
      - English Comment
  3. Parse vertices Information
  4. Parse Material Information
  5. Parse Bone Information
  6. Parse Morph Information
### WIP
  1. Implement Display trait
  2. Parse RigidBody Information
  3. Parse Joint
  4. Parse SoftBody
## How to Use
### 1. Import
```rust
extern crate PMXUtil;
use PMXUtil::pmx_loader::pmx_loader::PMXLoader;
```
### 2. Create loader instance and read  
```rust
let mut loader= PMXLoader::open("/path/to/pmxfile");
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
```
### 3 Create Writer instance and write
    this library always write text as UTF8 byte stream

```rust
        let mut writer =PMXWriter::begin_writer("/path/to/pmxfile");
        writer.set_model_info(Some(&model_info.name),Some(&model_info.name_en),Some(&model_info.comment),Some(&model_info.comment_en));
        writer.add_vertices(&vertices.vertices);
        writer.add_faces(&faces.faces);
        writer.add_textures(&textures.textures);
        writer.add_materials(&materials.materials);
        writer.add_bones(&bones.bones);
        writer.add_morphs(&morphs.morphs);
        PMXWriter::write(writer);
```
## Note 
 this crate under construction so outputed pmx file is not valid format because they don't have rigidbody section and joint section 
