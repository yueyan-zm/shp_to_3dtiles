# 简介
将shp转化成3dtiles白模的工具

# 用法说明
### 命令行格式  

`shp_to_3dtiles.exe [shp文件路径] [shp高度字段] [tif地形文件路径]`  

前两个参数必选，地形文件可不加

### 示例命令  

`shp_to_3dtiles.exe D:\ditu\test.shp height D:\ditu\test.tif`

# Reference
1.3dtiles https://github.com/fanvanzh/3dtiles  
2.Cesium3DTilesConverter https://github.com/scially/Cesium3DTilesConverter

# About Project  

这个项目参考了上面两个项目，因为原项目不支持地形文件，所以我就自己重写了中间shp转化的这个部分，并加上地形的支持。  
我也是一个rust的学习者，所以代码很多不规范，请见谅。
