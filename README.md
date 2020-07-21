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
### How to Use
1. Import
```
extern crate PMXUtil;
use PMXUtil::pmx_loader::pmx_loader::PMXLoader;
```
2. Create loader instance and read  
```rust
let mut loader=PMXLoader::open("/path/to/pmxfile");
let loader=PMXLoader::open("/path/to/pmxfile");
let (model_info,next_stage)=loader.read_pmx_model_info();
let (vertices,next_stage)=next_stage.read_pmx_vertices();

```


