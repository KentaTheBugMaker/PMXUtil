### PMXUtil
## A partial PMX loader and Writer written in Rust
### What can this crate do
  1. Parse PMX 2.0/2.1 header
  2. Parse PMX 2.0/2.1 Model Info
      - Name
      - English Name
      - Comment
      - English Comment
  3. Parse vertices Information
  4. Parse Material Information
  5. Parse Bone Information
  6. Parse Morph Information
  7. Parse Frame Information
  8. Parse Rigid Information
  9. parse Joint Information
  10. Parse SoftBody Information   
  11. Write PMX 2.0 header
  12. Write Model info 
  13. Write vertices
  14. Write Materials
  15. Write Bone
  16. Write Morph
  17. Write Frame
  18. Write Rigid
  19. Write Joint
### WIP
  1. Implement Display trait

## How to Use
### 1. Import
```rust
extern crate PMXUtil;
use PMXUtil::pmx_loader::PMXLoader;
```
### 2. Create loader instance and read  
```rust
let mut loader= PMXLoader::open("/path/to/pmxfile");
let header = loader.get_header();
println!("{:#?}", header);
let (model_info, ns) = loader.read_pmx_model_info();
print!("{:#?}", model_info);
let (vertices, ns) =ns.read_pmx_vertices();
print!("{}", vertices);
let (faces, ns) = ns.read_pmx_faces();
println!("{}", faces);
let (textures, ns) = ns.read_texture_list();
println!("{}", textures);
let (materials, ns) = ns.read_pmx_materials();
println!("{:#?}", materials);
```
### 3 Create Writer instance and write
    this library always write text as UTF8 byte stream

```rust
        let mut writer =PMXWriter::begin_writer("/path/to/pmxfile");
        writer.set_model_info(Some(&model_info.name),Some(&model_info.name_en),Some(&model_info.comment),Some(&model_info.comment_en));
        writer.add_vertices(&vertices);
        writer.add_faces(&faces);
        writer.add_textures(&textures);
        writer.add_materials(&materials);
        writer.add_bones(&bones);
        writer.add_morphs(&morphs);
        PMXWriter::write(writer);
```
## Note 
This crate is under construction so PMX 2.0 support is correct but PMX2.1 soft body is not supported.

 more example for https://github.com/t18b219k/PMXViewer_VK