use anyhow::anyhow;
use flate2::{self, read::ZlibDecoder};
use std::io::Read;
use std::string::ToString;

#[derive(Debug)]
struct GitObjectInfos {
    id: String,
    mode: String,
    path: String,
    r#type: String,
}

trait GitObject {
    fn from_buf(str: Vec<u8>) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

#[derive(Debug)]
struct GitCommit {
    infos: GitObjectInfos,
    raw: String,
}

// impl GitObject for GitCommit {
//     fn from_buf(buf: Vec<u8>) -> Result<Self, anyhow::Error> {
//         let mut decoder = ZlibDecoder::new(&buf[..]);
//         let mut content = String::new();
//         decoder.read_to_string(&mut content)?;
//         // strip blob header before null caracter
//         let [object_type, object_string] = content.splitn(2, '\0').collect::<Vec<&str>>()[..]
//         else {
//             return Err(anyhow!("Failed to extract object type"));
//         };
//         print!("!!!{} {}", blob_type, blob_string);
//         Ok(Self {
//             infos: GitObjectInfos {
//                 id: "toto_id".to_string(),
//                 mode: "toto_mode".to_string(),
//                 path: "toto_path".to_string(),
//                 r#type: "toto_type".to_string(),
//             },
//             raw: object_string.to_string(), // cleaned up part
//         })
//     }
// }

struct GitBlob {
    infos: GitObjectInfos,
}

struct GitTree {
    infos: GitObjectInfos,
}
