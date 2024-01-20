use anyhow::anyhow;
use flate2::{self, read::ZlibDecoder};
use nom::IResult;
use std::io::Read;

/// Decompress any git object into a Vec<u8>
pub fn unpack_object(buf: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = ZlibDecoder::new(&buf[..]);
    let mut content = Vec::new();
    decoder.read_to_end(&mut content)?;
    Ok(content)
}

/// Accepts the decompressed buffer and parses it
pub fn parse_object_buf(buf: Vec<u8>) -> Result<(), anyhow::Error> {
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

pub fn parse_object_header<'i, 'j>(buf: &'i [u8]) -> IResult<&'i [u8], (&'j str, usize)> {
    return Ok((&[12, 12], (std::str::from_utf8(b"hello").unwrap(), 2000)));
}

mod tests {
    use super::{parse_object_buf, parse_object_header, unpack_object};

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

    const GIT_COMMIT_BUFFER_UNPACKED: [u8; 259] = [
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
    fn test_unpack_object() {
        let input_buffer = Vec::from(GIT_COMMIT_BUFFER);
        assert_eq!(
            unpack_object(input_buffer).unwrap(),
            Vec::from(GIT_COMMIT_BUFFER_UNPACKED)
        )
    }

    #[test]
    fn test_parse_object_header_with_commit_object_type() {
        // node -e "console.log('commit 248'.split('').map(c => c.charCodeAt(0)))"
        let input_buffer: [u8; 10] = [99, 111, 109, 109, 105, 116, 32, 50, 52, 56]; // commit 248
        let tail: [u8; 0] = [];
        let object_type = std::str::from_utf8(b"commit").unwrap();
        let size: usize = 248;
        assert_eq!(
            parse_object_header(&input_buffer).unwrap(),
            (&tail.as_slice()[..], (object_type, size))
        );
    }

    // #[test]
    // fn git_commit_from_string() -> Result<(), anyhow::Error> {
    //     parse_object_buf(content);
    //     Ok(())
    // }
}
