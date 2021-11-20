# Header

|type|description |
|----|----------- |
|[u8;4]|magic number to validate file is pmx   |
|f32|pmx version |
|u8 |number of infomations|
|u8 | 0=>UTF-16LE,1=>UTF-8|
|u8 |number of additional uvs|
|u8 |size of vertex index 1,2 or 4|
|u8 |size of texture index 1,2 or 4|
|u8 |size of material index 1,2 or 4|
|u8 |size of bone index 1,2 or 4|
|u8 |size of morph index 1,2 or 4|
|u8|size of rigid index 1,2, or 4|
