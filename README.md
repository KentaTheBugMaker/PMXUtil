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
```
let mut loader=PMXLoader::open("/path/to/pmxfile");
//get _header information
let _header=loader.get_header();
//get model information returns Result<PMXModelInfo,()>
let _model_info=loader.read_pmx_model_info().unwrap();
```


