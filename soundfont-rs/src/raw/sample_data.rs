use crate::riff::Chunk;
use crate::{error::Error, riff::ChunkId};

use std::io::{Read, Seek};

/// Sample can be read like so:
/// ```ignore
/// file.seek(SeekFrom::Start(chunk.offset))?;
///
/// let mut buff = vec![0u8; chunk.len as usize];
/// file.read_exact(&mut buff)?;
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct SampleChunk {
    pub offset: u64,
    pub len: u32,
}

impl SampleChunk {
    fn new(chunk: Chunk) -> Self {
        Self {
            offset: chunk.content_offset(),
            len: chunk.len(),
        }
    }
}

/// The Sample Binary Data
#[derive(Debug)]
pub struct SampleData {
    /// The smpl sub-chunk, if present, contains one or more “samples” of digital audio information in the form of linearly coded sixteen bit, signed, little endian (least significant byte first) words.  Each sample is followed by a minimum of forty-six zero valued sample data points.  These zero valued data points are necessary to guarantee that any reasonable upward pitch shift using any reasonable interpolator can loop on zero data at the end of the sound.
    pub smpl: Option<SampleChunk>,
    /// The sm24 sub-chunk, if present, contains the least significant byte counterparts to each sample data point contained in the smpl chunk. Note this means for every two bytes in the [Self::smpl] sub-chunk there is a 1-byte counterpart in [Self::sm24] sub-chunk.
    ///
    /// These sample waveform points are to be combined with the sample waveform points in the smpl sub-chunk, to collectively create a single sample data pool with up to 24 bits of resolution.
    ///
    /// If the smpl Sub-chunk is not present, the sm24 sub-chunk should be ignored. If the ifil version of the format is less than thatwhich represents 2.04, the sm24 sub-chunk should be ignored. If the size of the sm24 chunk is not exactly equal to the ½ the size of the smpl chunk (+ 1 byte in the case that ½ the size of smpl chunk is an odd value), the sm24 sub-chunk should be ignored.  
    ///
    /// In any and all cases where the sm24 sub-chunk is ignored, the synthesizer should render only those samples contained within the smpl sub-chunk.
    pub sm24: Option<SampleChunk>,
}

impl SampleData {
    pub(crate) fn read<F: Read + Seek>(sdta: &Chunk, file: &mut F) -> Result<Self, Error> {
        assert_eq!(sdta.id(), ChunkId::LIST);
        assert_eq!(sdta.read_type(file)?, ChunkId::sdta);

        let mut smpl = None;
        let mut sm24 = None;

        let mut iter = sdta.iter();
        while let Some(ch) = iter.next(file) {
            let ch = ch?;
            let id = ch.id();

            match id {
                // The Digital Audio Samples for the upper 16 bits
                ChunkId::smpl => {
                    smpl = Some(SampleChunk::new(ch));
                }
                // The Digital Audio Samples for the lower 8 bits
                ChunkId::sm24 => {
                    sm24 = Some(SampleChunk::new(ch));
                }
                _ => {
                    return Err(Error::UnexpectedMemberOfSampleData(ch));
                }
            }
        }

        Ok(Self { smpl, sm24 })
    }
}
