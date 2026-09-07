//! Audio mixing and crossfading utilities.

/// Performs a linear crossfade between two PCM audio buffers.
/// Assuming 24kHz 16-bit mono PCM.
/// The transition occurs over `transition_samples` at the end of `pcm1` and the start of `pcm2`.
pub fn crossfade(pcm1: &[u8], pcm2: &[u8], transition_samples: usize) -> Vec<u8> {
    // 1. Convert byte slices to i16 samples (little-endian)
    let samples1: Vec<i16> = pcm1
        .as_chunks::<2>()
        .0
        .iter()
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    let samples2: Vec<i16> = pcm2
        .as_chunks::<2>()
        .0
        .iter()
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    if transition_samples == 0 {
        let mut result = Vec::with_capacity(pcm1.len() + pcm2.len());
        result.extend_from_slice(pcm1);
        result.extend_from_slice(pcm2);
        return result;
    }

    // Ensure we have enough samples for the transition
    let actual_transition = transition_samples.min(samples1.len()).min(samples2.len());

    let mut result_samples =
        Vec::with_capacity(samples1.len() + samples2.len() - actual_transition);

    // Part 1: pcm1 before the transition
    let pcm1_before_transition_end = samples1.len() - actual_transition;
    result_samples.extend_from_slice(&samples1[..pcm1_before_transition_end]);

    // Part 2: The crossfade transition
    // alpha goes from 0.0 (all pcm1) to 1.0 (all pcm2)
    for i in 0..actual_transition {
        let alpha = i as f32 / actual_transition as f32;
        let s1 = samples1[pcm1_before_transition_end + i];
        let s2 = samples2[i];

        // Linear crossfade: (1 - alpha) * s1 + alpha * s2
        let mixed = ((1.0 - alpha) * s1 as f32 + alpha * s2 as f32) as i16;
        result_samples.push(mixed);
    }

    // Part 3: pcm2 after the transition
    if samples2.len() > actual_transition {
        result_samples.extend_from_slice(&samples2[actual_transition..]);
    }

    // 4. Convert back to u8 bytes (little-endian)
    let mut result_bytes = Vec::with_capacity(result_samples.len() * 2);
    for sample in result_samples {
        result_bytes.extend_from_slice(&sample.to_le_bytes());
    }

    result_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossfade_basic() {
        // 4 samples each, 2 bytes per sample
        // s1: [1, 1, 1, 1] -> bytes: [1,0, 1,0, 1,0, 1,0]
        // s2: [2, 2, 2, 2] -> bytes: [2,0, 2,0, 2,0, 2,0]
        let pcm1 = vec![1, 0, 1, 0, 1, 0, 1, 0];
        let pcm2 = vec![2, 0, 2, 0, 2, 0, 2, 0];

        // Crossfade over 2 samples
        // Expected result:
        // pcm1[0..2] -> [1, 0, 1, 0] (samples 1, 1)
        // transition[0]: alpha=0.0 -> (1.0*1 + 0.0*2) = 1
        // transition[1]: alpha=0.5 -> (0.5*1 + 0.5*2) = 1.5 -> 1 (as i16)
        // pcm2[2..4] -> [2, 0, 2, 0] (samples 2, 2)
        // Total samples: 1, 1, 1, 1, 2, 2
        // Wait, if transition is 2:
        // samples1.len() = 4, samples2.len() = 4, transition = 2
        // pcm1_before = 4 - 2 = 2. samples1[0..2] = [1, 1]
        // transition:
        // i=0: alpha=0/2=0.0. s1=samples1[2]=1, s2=samples2[0]=2. mixed = 1*1 + 0*2 = 1
        // i=1: alpha=1/2=0.5. s1=samples1[3]=1, s2=samples2[1]=2. mixed = 0.5*1 + 0.5*2 = 1.5 -> 1
        // pcm2_after: samples2[2..4] = [2, 2]
        // Result samples: [1, 1, 1, 1, 2, 2]

        let result = crossfade(&pcm1, &pcm2, 2);
        let result_samples: Vec<i16> = result
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        assert_eq!(result_samples, vec![1, 1, 1, 1, 2, 2]);
    }
}
