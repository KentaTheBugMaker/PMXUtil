# The Structure of PMX files
 PMX files contains vertex buffer, index buffer ,texture path,bones,joints,etc.
 
We want to teach PMX structure to understand where the value passed.



|name|description |
|-----|------|
|header|contains pmx version,text encoding,etc. let see [header](#Header)|
|model info|contains name ,etc. let see [model info](#ModelInfo)|
|number of vertices|don't care|
|vertices|contains position,normal,uv,etc. let see [vertex](#Vertex)|
|number of indices|don't care|
|indices|3 indices represent 1 triangle|
|number of textures|don't care|
|textures|path of texture relative to PMX file|
|number of materials|don't care|
|materials|contains colors ,texture index,etc|
|number of morphs|don't care|
|morphs|contains|
|number of frame|don't care|
|frames|contains|
|number of rigid bodies|don't care|
|rigid bodies|contains|
|number of joints|don't care|
|joints|contains |

#Header

|type|description |
|----|----------- |
|[u8;4]|magic number to validate file is pmx   |
|f32|pmx version 2.0 or 2.1|
|u8 |number of infomations|
|u8 | 0=>UTF-16LE,1=>UTF-8|
|u8 |number of additional uvs 0~4|
|u8 |size of vertex index 1,2 or 4|
|u8 |size of texture index 1,2 or 4|
|u8 |size of material index 1,2 or 4|
|u8 |size of bone index 1,2 or 4|
|u8 |size of morph index 1,2 or 4|
|u8|size of rigid index 1,2, or 4|

#ModelInfo

|type|description|
|----|-----------|
|Textbuf|model name in japanese|
|Textbuf|model name in english|
|Textbuf|comment in japanese|
|Textbuf|comment in english|

#Vertex 

|type|description|
|----|-----------|
|[f32;3]| position|
|[f32;3]| normal|
|[f32;2]|UV in directX coordinate system|
|[[f32;4]]|additional uv parameter|
|u8|weight kind|
|WeightKinds|deforming information. let see [WeightKinds](#WeightKinds)|

#WeightKinds

## BDEF 1
 a bone with weight 1.0.

|type|description|
|----|-----------|
|Index|bone index to refer|

## BDEF 2
 two bone with normalized weights.

|type|description|
|----|-----------|
|Index|bone 1 index to refer|
|Index|bone 2 index to refer|
|f32|weight of bone 1|

## BDEF 4
 4 bone without guaranty of normalized weights.

|type|description|
|----|-----------|
|Index|bone 1 index to refer|
|Index|bone 2 index to refer|
|Index|bone 3 index to refer|
|Index|bone 4 index to refer|
|f32|weight of bone 1|
|f32|weight of bone 2|
|f32|weight of bone 3|
|f32|weight of bone 4|

## SDEF
 spherical deform bones.

 |type|description|
 |----|-----------|
 |Index|bone 1 index to refer|
 |Index|bone 2 index to refer|
 |f32|weight of bone 1|
 |[f32;3]|SDEF-C|
 |[f32;3]|SDEF-R0|
 |[f32;3]|SDEF-R1|
 
## QDEF (2.1) extension
DualQuaternion weight deform.

|type|description|
|----|-----------|
|Index|bone 1 index to refer|
|Index|bone 2 index to refer|
|Index|bone 3 index to refer|
|Index|bone 4 index to refer|
|f32|weight of bone 1|
|f32|weight of bone 2|
|f32|weight of bone 3|
|f32|weight of bone 4|