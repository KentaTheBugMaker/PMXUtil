### PMXUtil
## A PMX reader and writer written in Rust
### What can this crate do
all of pmx related jobs.
### Conformance Test
PMX 2.0 I/O tested by
 1. read original file and write it to another file
 2. compare these file  by `cargo test`
 3. compare these file by WinMerge 
### WIP
  * improving docs

## How to Use

###  Create reader instance and read  

```rust
let mut loader= ModelInfoStage::open("/path/to/pmxfile").unwrap();
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

### Create Writer instance and write
    you can choose text encoding UTF-8 or UTF-16LE but MMD only support UTF-16LE.

```rust

        use pmx_util::writer::Writer;
        let mut writer =Writer::begin_writer("/path/to/pmxfile").unwrap();
        writer.set_model_info(
            &ModelInfo{
                name:"A Model Name in your local language".to_owned()
                name_en:"A Model Name in english".to_owned()
                comment:"Comment in you local language".to_owned()
                comment_en:"Comment in english".to_owned()
            }
        );
        writer.add_vertices(&vertices);
        writer.add_faces(&faces);
        writer.add_textures(&textures);
        writer.add_materials(&materials);
        writer.add_bones(&bones);
        writer.add_morphs(&morphs);
        Writer::write(writer);
```
## Note 

 more example for https://github.com/t18b219k/n_pmx_viewer
 