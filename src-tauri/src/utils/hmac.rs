use sha2::{Digest, Sha256};

const BLOCK: usize = 64;

pub fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    let mut block = [0u8; BLOCK];
    if key.len() > BLOCK {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let inner: Vec<u8> = block
        .iter()
        .map(|b| b ^ 0x36)
        .chain(message.iter().copied())
        .collect();
    let inner_hash = Sha256::digest(&inner);
    let outer: Vec<u8> = block
        .iter()
        .map(|b| b ^ 0x5c)
        .chain(inner_hash.iter().copied())
        .collect();
    Sha256::digest(&outer)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_matches_the_rfc_4231_vectors() {
        assert_eq!(
            hmac_sha256_hex(b"Jefe", b"what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        assert_eq!(
            hmac_sha256_hex(&[0x0b; 20], b"Hi There"),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        let long_key = [0xaa; 131];
        assert_eq!(
            hmac_sha256_hex(
                &long_key,
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            ),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }
}
