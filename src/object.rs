use anyhow::anyhow;
use flate2::{self, read::ZlibDecoder};
use std::io::Read;
use std::string::ToString;

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

struct GitCommit {
    infos: GitObjectInfos,
    raw: String,
}

impl GitObject for GitCommit {
    fn from_buf(str: Vec<u8>) -> Result<Self, anyhow::Error> {
        let mut decoder = ZlibDecoder::new(&str[..]);
        let mut content = String::new();
        decoder.read_to_string(&mut content)?;
        // strip blob header before null caracter
        let blob_string = content.splitn(2, '\0').collect::<Vec<&str>>()[1];
        print!("{}", blob_string);
        Ok(Self {
            infos: GitObjectInfos {
                id: "toto_id".to_string(),
                mode: "toto_mode".to_string(),
                path: "toto_path".to_string(),
                r#type: "toto_type".to_string(),
            },
            raw: blob_string.to_string(), // cleaned up part
        })
    }
}

struct GitBlob {
    infos: GitObjectInfos,
}

struct GitTree {
    infos: GitObjectInfos,
}

#[cfg(test)]
mod tests {
    use crate::object::GitObject;

    use super::GitCommit;

    #[test]
    fn git_commit_from_string() -> Result<(), anyhow::Error> {
        let content = r#"���=��U��<ߐ�u���L_�΃8��(��1���߷�M��Uv�0�E�[�\�����$sSS���Ңx`��g0LZ��j���\��gOμW��7>��PE MY|JjRi:Hq��?�=��m>^������]�yp��"#.to_string();
        let content = vec![
            120, 1, 173, 206, 77, 106, 3, 49, 12, 134, 225, 174, 125, 10, 45, 211, 22, 130, 252,
            23, 205, 132, 80, 10, 189, 65, 111, 32, 91, 114, 103, 2, 19, 7, 219, 129, 30, 191, 67,
            23, 61, 65, 183, 15, 47, 31, 95, 174, 219, 182, 14, 112, 132, 79, 163, 169, 2, 229, 20,
            188, 243, 138, 101, 102, 79, 81, 38, 9, 20, 78, 105, 10, 196, 182, 40, 82, 12, 228, 37,
            166, 108, 238, 220, 244, 54, 32, 107, 33, 14, 193, 250, 104, 253, 36, 214, 57, 141, 49,
            164, 57, 248, 178, 103, 49, 57, 117, 69, 120, 62, 89, 195, 143, 177, 212, 6, 31, 75,
            91, 251, 168, 247, 69, 225, 179, 246, 174, 3, 46, 249, 143, 142, 237, 151, 222, 179,
            126, 169, 172, 219, 49, 215, 237, 13, 44, 97, 180, 20, 17, 9, 94, 209, 34, 154, 93,
            247, 203, 67, 255, 101, 204, 20, 229, 113, 120, 121, 62, 3, 139, 192, 181, 67, 231,
            155, 164, 250, 109, 126, 0, 175, 12, 87, 215,
        ];
        let git_commit = GitCommit::from_buf(content)?;
        // println!("{}", GitCommit::from_buf(content));
        Ok(())
    }
}
