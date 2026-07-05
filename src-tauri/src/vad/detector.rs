use std::collections::VecDeque;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VadDetector {
    threshold: f32,
    frame_size: usize,
    energy_history: VecDeque<f32>,
    history_size: usize,
    is_speaking: bool,
    silence_frames: usize,
    max_silence_frames: usize,
}

impl VadDetector {
    pub fn new(threshold: f32, frame_size: usize) -> Self {
        Self {
            threshold,
            frame_size,
            energy_history: VecDeque::with_capacity(10),
            history_size: 10,
            is_speaking: false,
            silence_frames: 0,
            max_silence_frames: 3, // ~9 seconds with 3-second chunks, ~1 second with 1-second chunks
        }
    }

    pub fn detect(&mut self, audio: &[f32]) -> VadResult {
        let energy = self.calculate_energy(audio);

        self.energy_history.push_back(energy);
        if self.energy_history.len() > self.history_size {
            self.energy_history.pop_front();
        }

        let avg_energy = self.average_energy();
        let is_voice = energy > self.threshold && energy > avg_energy * 0.5;

        if is_voice {
            self.is_speaking = true;
            self.silence_frames = 0;
        } else if self.is_speaking {
            self.silence_frames += 1;
            if self.silence_frames >= self.max_silence_frames {
                self.is_speaking = false;
                self.silence_frames = 0;
            }
        }

        VadResult {
            is_speaking: self.is_speaking,
            energy,
            threshold: self.threshold,
        }
    }

    fn calculate_energy(&self, audio: &[f32]) -> f32 {
        if audio.is_empty() {
            return 0.0;
        }

        let sum: f32 = audio.iter().map(|&s| s * s).sum();
        (sum / audio.len() as f32).sqrt()
    }

    fn average_energy(&self) -> f32 {
        if self.energy_history.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.energy_history.iter().sum();
        sum / self.energy_history.len() as f32
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    pub fn set_max_silence_frames(&mut self, max: usize) {
        self.max_silence_frames = max;
    }

    pub fn reset(&mut self) {
        self.is_speaking = false;
        self.silence_frames = 0;
        self.energy_history.clear();
    }
}

impl Default for VadDetector {
    fn default() -> Self {
        Self::new(0.01, 1600)
    }
}

#[derive(Debug, Clone)]
pub struct VadResult {
    pub is_speaking: bool,
    pub energy: f32,
    pub threshold: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic detection ───────────────────────────────────────────

    #[test]
    fn test_silence_detection() {
        let mut vad = VadDetector::new(0.01, 1600);
        let silence = vec![0.0; 1600];
        let result = vad.detect(&silence);
        assert!(!result.is_speaking);
    }

    #[test]
    fn test_voice_detection() {
        let mut vad = VadDetector::new(0.01, 1600);
        let voice: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let result = vad.detect(&voice);
        assert!(result.is_speaking);
    }

    #[test]
    fn test_empty_audio() {
        let mut vad = VadDetector::new(0.01, 1600);
        let result = vad.detect(&[]);
        assert!(!result.is_speaking);
        assert_eq!(result.energy, 0.0);
    }

    // ── Energy calculation ────────────────────────────────────────

    #[test]
    fn test_energy_zero_for_silence() {
        let mut vad = VadDetector::new(0.01, 1600);
        let result = vad.detect(&vec![0.0; 100]);
        assert_eq!(result.energy, 0.0);
    }

    #[test]
    fn test_energy_positive() {
        let mut vad = VadDetector::new(0.01, 1600);
        let audio = vec![0.5; 100];
        let result = vad.detect(&audio);
        assert!(result.energy > 0.0);
    }

    #[test]
    fn test_energy_rms_calculation() {
        let mut vad = VadDetector::new(0.01, 1600);
        // RMS of [1.0, 1.0, 1.0] = sqrt(1) = 1.0
        let result = vad.detect(&[1.0, 1.0, 1.0]);
        assert!((result.energy - 1.0).abs() < 0.001);
    }

    // ── Threshold behavior ────────────────────────────────────────

    #[test]
    fn test_below_threshold_not_speaking() {
        let mut vad = VadDetector::new(0.5, 1600);
        // Low energy audio (below threshold)
        let audio = vec![0.01; 1600];
        let result = vad.detect(&audio);
        assert!(!result.is_speaking);
    }

    #[test]
    fn test_above_threshold_speaking() {
        let mut vad = VadDetector::new(0.01, 1600);
        let audio: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let result = vad.detect(&audio);
        assert!(result.is_speaking);
    }

    #[test]
    fn test_set_threshold() {
        let mut vad = VadDetector::new(0.01, 1600);
        vad.set_threshold(0.5);
        assert_eq!(vad.threshold, 0.5);
    }

    // ── Silence frames / hysteresis ───────────────────────────────

    #[test]
    fn test_silence_frames_count() {
        let mut vad = VadDetector::new(0.01, 1600);
        // Start speaking
        let voice: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        vad.detect(&voice);
        assert!(vad.is_speaking);

        // Go silent — should still be speaking (silence_frames < max)
        let silence = vec![0.0; 1600];
        for _ in 0..2 {
            vad.detect(&silence);
            assert!(vad.is_speaking, "Should still be speaking during silence ramp");
        }
    }

    #[test]
    fn test_silence_frames_stop_speaking() {
        let mut vad = VadDetector::new(0.01, 1600);
        // Start speaking
        let voice: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        vad.detect(&voice);
        assert!(vad.is_speaking);

        // Go silent for max_silence_frames (3)
        let silence = vec![0.0; 1600];
        for _ in 0..3 {
            vad.detect(&silence);
        }
        assert!(!vad.is_speaking, "Should stop speaking after enough silence");
    }

    #[test]
    fn test_speech_resets_silence_counter() {
        let mut vad = VadDetector::new(0.01, 1600);
        // Start speaking
        let voice: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        vad.detect(&voice);
        assert!(vad.is_speaking);

        // Some silence
        let silence = vec![0.0; 1600];
        vad.detect(&silence);
        vad.detect(&silence);

        // Speech again — resets silence counter
        vad.detect(&voice);
        assert!(vad.is_speaking);

        // Now 2 silences — should still be speaking (max is 3)
        for _ in 0..2 {
            vad.detect(&silence);
            assert!(vad.is_speaking);
        }
    }

    // ── Reset ─────────────────────────────────────────────────────

    #[test]
    fn test_reset() {
        let mut vad = VadDetector::new(0.01, 1600);
        // Start speaking
        let voice: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        vad.detect(&voice);
        assert!(vad.is_speaking);

        vad.reset();
        assert!(!vad.is_speaking);
        assert_eq!(vad.silence_frames, 0);
        assert!(vad.energy_history.is_empty());
    }

    #[test]
    fn test_reset_clears_energy_history() {
        let mut vad = VadDetector::new(0.01, 1600);
        let audio = vec![0.5; 1600];
        for _ in 0..5 {
            vad.detect(&audio);
        }
        assert!(!vad.energy_history.is_empty());

        vad.reset();
        assert!(vad.energy_history.is_empty());
    }

    // ── Default ───────────────────────────────────────────────────

    #[test]
    fn test_default_values() {
        let vad = VadDetector::default();
        assert_eq!(vad.threshold, 0.01);
        assert_eq!(vad.frame_size, 1600);
        assert!(!vad.is_speaking);
        assert_eq!(vad.silence_frames, 0);
        assert_eq!(vad.max_silence_frames, 3);
    }

    // ── VadResult ─────────────────────────────────────────────────

    #[test]
    fn test_result_contains_energy() {
        let mut vad = VadDetector::new(0.01, 1600);
        let result = vad.detect(&[0.5; 100]);
        assert!(result.energy > 0.0);
        assert_eq!(result.threshold, 0.01);
    }

    #[test]
    fn test_result_clone() {
        let mut vad = VadDetector::new(0.01, 1600);
        let result = vad.detect(&[0.5; 100]);
        let cloned = result.clone();
        assert_eq!(result.is_speaking, cloned.is_speaking);
        assert_eq!(result.energy, cloned.energy);
    }

    // ── Energy history ────────────────────────────────────────────

    #[test]
    fn test_energy_history_limited_to_10() {
        let mut vad = VadDetector::new(0.01, 1600);
        let audio = vec![0.5; 1600];
        for _ in 0..20 {
            vad.detect(&audio);
        }
        assert!(vad.energy_history.len() <= 10);
    }

    #[test]
    fn test_average_energy_used_for_detection() {
        let mut vad = VadDetector::new(0.01, 1600);
        // First frames: low energy to build baseline
        let low = vec![0.001; 1600];
        for _ in 0..10 {
            vad.detect(&low);
        }
        // Now slightly above threshold but below avg*0.5
        let medium = vec![0.02; 1600];
        let result = vad.detect(&medium);
        // Should not be speaking because energy < avg_energy * 0.5
        // (avg is ~0.001, so avg*0.5 = 0.0005, and 0.02 > 0.0005)
        // Actually 0.02 > 0.01 (threshold) AND 0.02 > 0.0005 → speaking
        assert!(result.is_speaking);
    }
}
