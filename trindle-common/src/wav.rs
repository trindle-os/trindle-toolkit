//! A mono, 16-bit WAV file for a run of samples.

pub fn wav(samples: &[i16], rate: u32) -> Option<Vec<u8>> {
    let data = u32::try_from(samples.len() * 2).ok().filter(|d| *d <= u32::MAX - 36)?;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&rate.checked_mul(2)?.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    out.extend(samples.iter().flat_map(|s| s.to_le_bytes()));
    Some(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_header_says_what_follows() {
        let file = super::wav(&[1, -2, 3], 16000).unwrap();
        assert_eq!(&file[..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(file[4..8].try_into().unwrap()), 36 + 6);
        assert_eq!(u32::from_le_bytes(file[24..28].try_into().unwrap()), 16000);
        assert_eq!(u32::from_le_bytes(file[40..44].try_into().unwrap()), 6);
        assert_eq!(file.len(), 44 + 6);
        assert_eq!(&file[44..46], &1i16.to_le_bytes());
    }
}
