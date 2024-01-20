use anyhow::anyhow;
use flate2::{self, read::ZlibDecoder};
use std::io::Read;
use std::string::ToString;

fn parse_object_buf(buf: Vec<u8>) -> Result<(), anyhow::Error> {
    let mut decoder = ZlibDecoder::new(&buf[..]);
    let mut content = String::new();
    decoder.read_to_string(&mut content)?;
    // strip blob header before null caracter
    let [object_type, object_string] = content.splitn(2, '\0').collect::<Vec<&str>>()[..] else {
        return Err(anyhow!("Failed to extract object type"));
    };
    print!("{} {}", object_type, object_string);
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use crate::object::GitObject;

    use super::{parse_object_buf, GitCommit};

    #[test]
    fn git_commit_from_string() -> Result<(), anyhow::Error> {
        let content = vec![
            120, 1, 149, 142, 77, 10, 194, 48, 16, 133, 93, 231, 20, 179, 244, 7, 100, 210, 38, 77,
            34, 34, 130, 11, 151, 130, 55, 72, 50, 19, 90, 161, 70, 218, 241, 254, 22, 241, 2, 110,
            30, 239, 189, 197, 199, 151, 235, 56, 14, 2, 141, 241, 43, 153, 152, 65, 59, 167, 57,
            33, 114, 140, 75, 165, 96, 200, 53, 166, 132, 76, 77, 73, 209, 115, 23, 60, 7, 87, 172,
            86, 175, 56, 241, 83, 0, 173, 54, 173, 165, 148, 2, 26, 239, 177, 43, 156, 49, 19, 179,
            201, 134, 200, 99, 118, 158, 60, 167, 214, 170, 248, 150, 190, 78, 112, 233, 167, 97,
            150, 250, 234, 25, 238, 117, 158, 89, 224, 248, 93, 231, 111, 142, 241, 185, 207, 117,
            60, 45, 26, 104, 187, 208, 6, 236, 96, 135, 26, 81, 45, 239, 226, 41, 252, 55, 193,
            234, 31, 65, 21, 142, 178, 222, 110, 14, 16, 137, 224, 58, 200, 45, 61, 56, 203, 172,
            62, 134, 170, 80, 70,
        ];
        parse_object_buf(content);
        Ok(())
    }
}
