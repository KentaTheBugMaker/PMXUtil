# Header

the description of raw header.

pmx header is written in this order.

|type|description |
|----|----------- |
|[u8;4]|magic number to validate file is pmx if this item is not `[0x50,0x4d,0x58,0x20]` you should stop reading.|
|f32|pmx version 2.0 or 2.1 is allowed|
|u8 |number of following information|
|u8 | 0=>UTF-16LE,1=>UTF-8|
|u8 |number of additional uvs 0,1,2,3 or 4 is allowed |
|u8 |size of vertex index 1,2 or 4|
|u8 |size of texture index 1,2 or 4|
|u8 |size of material index 1,2 or 4|
|u8 |size of bone index 1,2 or 4|
|u8 |size of morph index 1,2 or 4|
|u8 |size of rigid index 1,2, or 4|

