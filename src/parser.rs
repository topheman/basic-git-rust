use anyhow::anyhow;
use flate2::{self, read::ZlibDecoder};
use nom::{bytes::complete::take_till, IResult, Parser};
use std::io::Read;

#[derive(PartialEq, Debug)]
pub enum GitObjectHeader {
    Commit(usize),
    Tree(usize),
    Blob(usize),
}

pub struct GitObject<'i> {
    pub header: GitObjectHeader,
    pub raw_data: &'i [u8],
}

impl GitObjectHeader {
    /// Creates a GitObjectHeader from a Vec<u8> containing both the type and length
    pub fn from_vec(vec: &[u8]) -> Result<Self, anyhow::Error> {
        let result = split_at_code(32).parse(&vec);
        match result {
            Ok(([], (object_type, object_length))) => {
                // let foo = std::str::from_utf8(object_length)
                //     .and_then(|foo| foo.parse::<u32>().and_then(|res| Ok(res, ())));
                let object_length = std::str::from_utf8(object_length)?;
                let object_length = object_length.parse::<u32>()?;
                match object_type {
                    &[99, 111, 109, 109, 105, 116] => {
                        return Ok(GitObjectHeader::Commit(object_length as usize));
                    }
                    &[116, 114, 101, 101] => {
                        return Ok(GitObjectHeader::Tree(object_length as usize));
                    }
                    &[98, 108, 111, 98] => {
                        return Ok(GitObjectHeader::Blob(object_length as usize));
                    }
                    _ => {
                        return Err(anyhow!("[GitObjectHeader] Unsupported object type"));
                    }
                }
            }
            Ok((_, (_, _))) => {
                return Err(anyhow!(
                    "[GitObjectHeader] Parse error, still data left after content type content length"
                ));
            }
            Err(_) => {
                return Err(anyhow!("[GitObjectHeader] parse error"));
            }
        }
    }
}

impl GitObject<'_> {
    /// Creates a GitObject from a Vec<u8> containing de decompressed buffer of the git object
    pub fn from_vec(vec: &'static [u8]) -> Result<Self, anyhow::Error> {
        // todo is it possible not to leak ? due to parser::split_at_code
        // let owned_vec = vec.clone().to_owned().as_slice();
        let (_, (git_object_infos, git_object_raw_data)) = split_at_code(0).parse(vec)?;
        // std::mem::drop(owned_vec);
        println!("git_object_infos: {:?}", git_object_infos);
        let git_object_header = GitObjectHeader::from_vec(&git_object_infos)?;
        println!("git_object_header: {:?}", git_object_header);
        Ok(GitObject {
            header: git_object_header,
            raw_data: git_object_raw_data,
        })
    }
}

/// Decompress any git object into a Vec<u8>
pub fn decompress_object(buf: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = ZlibDecoder::new(&buf[..]);
    let mut content = Vec::new();
    decoder.read_to_end(&mut content)?;
    Ok(content)
}

/// Splits a Vec<u8> at a specific code.
///
/// ```
/// let input_buffer = [1, 2, 3, 4, 32, 5, 6, 7, 8, 9].as_slice();
/// let result = split_at_code(32).parse(input_buffer).unwrap();
/// assert_eq!(result, ([].as_slice(), ([1, 2, 3, 4].as_slice(), [5, 6, 7, 8, 9].as_slice())) );
/// ```
///
pub fn split_at_code<'i>(
    code: u8,
) -> impl Parser<&'i [u8], (&'i [u8], &'i [u8]), nom::error::Error<&'i [u8]>> {
    let match_code = move |num: u8| num == code;
    move |input: &'i [u8]| -> IResult<&'i [u8], (&'i [u8], &'i [u8]), nom::error::Error<&'i [u8]>> {
        let (tail, part1) = take_till(match_code)(input)?;
        let (tail, part2) = take_till(match_code)(&tail[1..])?;
        return Ok((tail, (part1, part2)));
    }
}

#[cfg(test)]
mod tests {
    // imports and const are marked as unused because of tests

    use nom::Parser;

    use super::{decompress_object, split_at_code, GitObjectHeader};

    const GIT_COMMIT_BUFFER: [u8; 178] = [
        120, 1, 149, 142, 77, 10, 194, 48, 16, 133, 93, 231, 20, 179, 244, 7, 100, 210, 38, 77, 34,
        34, 130, 11, 151, 130, 55, 72, 50, 19, 90, 161, 70, 218, 241, 254, 22, 241, 2, 110, 30,
        239, 189, 197, 199, 151, 235, 56, 14, 2, 141, 241, 43, 153, 152, 65, 59, 167, 57, 33, 114,
        140, 75, 165, 96, 200, 53, 166, 132, 76, 77, 73, 209, 115, 23, 60, 7, 87, 172, 86, 175, 56,
        241, 83, 0, 173, 54, 173, 165, 148, 2, 26, 239, 177, 43, 156, 49, 19, 179, 201, 134, 200,
        99, 118, 158, 60, 167, 214, 170, 248, 150, 190, 78, 112, 233, 167, 97, 150, 250, 234, 25,
        238, 117, 158, 89, 224, 248, 93, 231, 111, 142, 241, 185, 207, 117, 60, 45, 26, 104, 187,
        208, 6, 236, 96, 135, 26, 81, 45, 239, 226, 41, 252, 55, 193, 234, 31, 65, 21, 142, 178,
        222, 110, 14, 16, 137, 224, 58, 200, 45, 61, 56, 203, 172, 62, 134, 170, 80, 70,
    ];

    const GIT_COMMIT_BUFFER_UNCOMPRESSED: [u8; 259] = [
        99, 111, 109, 109, 105, 116, 32, 50, 52, 56, 0, 116, 114, 101, 101, 32, 49, 55, 55, 49,
        101, 98, 48, 48, 101, 97, 97, 49, 55, 55, 100, 57, 52, 100, 55, 50, 52, 102, 57, 99, 100,
        50, 102, 98, 97, 56, 101, 54, 57, 56, 101, 57, 55, 102, 53, 49, 10, 112, 97, 114, 101, 110,
        116, 32, 48, 53, 49, 52, 51, 53, 100, 98, 98, 57, 48, 52, 56, 56, 48, 54, 102, 101, 99, 48,
        99, 100, 101, 101, 52, 99, 52, 100, 100, 56, 48, 99, 55, 56, 100, 56, 101, 98, 51, 53, 10,
        97, 117, 116, 104, 111, 114, 32, 67, 104, 114, 105, 115, 116, 111, 112, 104, 101, 32, 82,
        111, 115, 115, 101, 116, 32, 60, 116, 111, 112, 104, 101, 64, 116, 111, 112, 104, 101, 109,
        97, 110, 46, 99, 111, 109, 62, 32, 49, 55, 48, 53, 54, 57, 51, 57, 48, 54, 32, 43, 48, 49,
        48, 48, 10, 99, 111, 109, 109, 105, 116, 116, 101, 114, 32, 67, 104, 114, 105, 115, 116,
        111, 112, 104, 101, 32, 82, 111, 115, 115, 101, 116, 32, 60, 116, 111, 112, 104, 101, 64,
        116, 111, 112, 104, 101, 109, 97, 110, 46, 99, 111, 109, 62, 32, 49, 55, 48, 53, 54, 57,
        51, 57, 53, 49, 32, 43, 48, 49, 48, 48, 10, 10, 102, 101, 97, 116, 40, 42, 41, 58, 32, 97,
        100, 100, 32, 71, 105, 116, 79, 98, 106, 101, 99, 116, 115, 10,
    ];

    #[test]
    fn test_split_at_code_one_code() {
        let input_buffer = [1, 2, 3, 4, 32, 5, 6, 7, 8, 9].as_slice();
        let tail = [].as_slice();
        let part1 = [1, 2, 3, 4].as_slice();
        let part2 = [5, 6, 7, 8, 9].as_slice();
        assert_eq!(
            split_at_code(32).parse(input_buffer).unwrap(),
            (tail, (part1, part2))
        )
    }

    #[test]
    fn test_split_at_code_multi_code() {
        let input_buffer = [1, 2, 3, 4, 32, 5, 6, 7, 8, 9, 32, 10, 11, 12].as_slice();
        let tail = [32, 10, 11, 12].as_slice(); // the tail is not empty if there are 32 more than once
        let part1 = [1, 2, 3, 4].as_slice();
        let part2 = [5, 6, 7, 8, 9].as_slice();
        assert_eq!(
            split_at_code(32).parse(input_buffer).unwrap(),
            (tail, (part1, part2))
        )
    }

    #[test]
    fn test_decompress_object() {
        let input_buffer = Vec::from(GIT_COMMIT_BUFFER);
        assert_eq!(
            decompress_object(input_buffer).unwrap(),
            Vec::from(GIT_COMMIT_BUFFER_UNCOMPRESSED)
        )
    }

    #[test]
    fn test_create_git_object_header_commit() {
        let input_buffer: [u8; 10] = [99, 111, 109, 109, 105, 116, 32, 50, 52, 56]; // commit 248
        let result = GitObjectHeader::from_vec(&input_buffer).unwrap();
        assert_eq!(result, GitObjectHeader::Commit(248));
    }

    #[test]
    fn test_create_git_object_header_tree() {
        let input_buffer: [u8; 8] = [116, 114, 101, 101, 32, 51, 56, 50]; // tree 385
        let result = GitObjectHeader::from_vec(&input_buffer).unwrap();
        assert_eq!(result, GitObjectHeader::Tree(382));
    }

    #[test]
    fn test_create_git_object_header_blob() {
        let input_buffer: [u8; 11] = [98, 108, 111, 98, 32, 50, 52, 56, 51, 56, 50]; // blob 248385
        let result = GitObjectHeader::from_vec(&input_buffer).unwrap();
        assert_eq!(result, GitObjectHeader::Blob(248382));
    }
}
