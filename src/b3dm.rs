use byteorder::{LittleEndian, WriteBytesExt};
use gltf::Error;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::io;

fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(C)]
pub struct FeatureTable {
    #[serde(rename = "BATCH_LENGTH")]
    pub batch_length: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(C)]
pub struct BatchTable {
    #[serde(rename = "batchId")]
    pub batch_id: Vec<u32>,
    pub height: Vec<f32>,
    pub name: Vec<String>,
}
#[repr(C)]
pub struct MakeB3dm<'a> {
    pub feature_table_json_byte_length: usize,
    pub batch_table_json_byte_length: usize,
    pub glb: Option<Cow<'a, [u8]>>,
}

impl<'a> MakeB3dm<'a> {
    //生成b3dm文件
    pub fn to_writer<W>(
        &self,
        mut writer: W,
        feature_string_vec: Vec<u8>,
        table_string_vec: Vec<u8>,
    ) -> Result<(), Error>
    where
        W: io::Write,
    {
        // 写入b3dm文件头部
        {
            let magic = b"b3dm";
            let version = 1;
            let feature_table_json_byte_length = self.feature_table_json_byte_length;
            //此处b3dm的二进制为空，即为0
            let feature_table_binary_byte_length = 0;
            let batch_table_json_byte_length = self.batch_table_json_byte_length;
            //此处b3dm的二进制为空，即为0
            let batch_table_binary_byte_length = 0;
            let mut length = (28
                + feature_table_json_byte_length
                + feature_table_binary_byte_length
                + batch_table_json_byte_length
                + batch_table_binary_byte_length) as usize;
            align_to_multiple_of_four(&mut length);

            if let Some(bin) = self.glb.as_ref() {
                length += bin.len();

                align_to_multiple_of_four(&mut length);
            }

            writer.write_all(&magic[..])?;
            writer.write_u32::<LittleEndian>(version)?;
            writer.write_u32::<LittleEndian>(length as u32)?;
            writer.write_u32::<LittleEndian>(feature_table_json_byte_length as u32)?;
            writer.write_u32::<LittleEndian>(feature_table_binary_byte_length as u32)?;
            writer.write_u32::<LittleEndian>(batch_table_json_byte_length as u32)?;
            writer.write_u32::<LittleEndian>(batch_table_binary_byte_length as u32)?;
        }

        // Write JSON chunk header
        {
            let mut length1 = feature_string_vec.len();
            align_to_multiple_of_four(&mut length1);
            let padding = length1 - feature_string_vec.len();

            writer.write_all(&feature_string_vec)?;
            for _ in 0..padding {
                writer.write_u8(0x20)?;
            }
        }

        {
            let mut length2 = table_string_vec.len();
            align_to_multiple_of_four(&mut length2);
            let padding = length2 - table_string_vec.len();

            writer.write_all(&table_string_vec)?;
            for _ in 0..padding {
                writer.write_u8(0x20)?;
            }
        }

        if let Some(bin) = self.glb.as_ref() {
            let mut length = bin.len();
            align_to_multiple_of_four(&mut length);
            let padding = length - bin.len();

            // writer.write_u32::<LittleEndian>(length as u32)?;
            writer.write_all(bin)?;
            for _ in 0..padding {
                writer.write_u8(0)?;
            }
        }

        Ok(())
    }
}
