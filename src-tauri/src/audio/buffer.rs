use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RingBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, data: &[f32]) {
        if self.capacity == 0 {
            return;
        }
        for &sample in data {
            if self.buffer.len() >= self.capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(sample);
        }
    }

    pub fn drain(&mut self) -> Vec<f32> {
        self.buffer.drain(..).collect()
    }

    pub fn drain_to(&mut self, count: usize) {
        let to_drain = count.min(self.buffer.len());
        self.buffer.drain(..to_drain);
    }

    pub fn take(&mut self, count: usize) -> Vec<f32> {
        let to_take = count.min(self.buffer.len());
        self.buffer.drain(..to_take).collect()
    }

    pub fn peek(&self, len: usize) -> Vec<f32> {
        self.buffer.iter().take(len).copied().collect()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new(16000 * 30) // 30 seconds at 16kHz
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic operations ──────────────────────────────────────────

    #[test]
    fn test_new_buffer() {
        let buf = RingBuffer::new(100);
        assert_eq!(buf.capacity(), 100);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_default_capacity() {
        let buf = RingBuffer::default();
        assert_eq!(buf.capacity(), 16000 * 30);
    }

    #[test]
    fn test_push_single_sample() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[42.0]);
        assert_eq!(buf.len(), 1);
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_push_multiple_samples() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0]);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_drain_returns_all_and_clears() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0]);
        let data = buf.drain();
        assert_eq!(data, vec![1.0, 2.0, 3.0]);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_drain_empty_buffer() {
        let mut buf = RingBuffer::new(10);
        let data = buf.drain();
        assert!(data.is_empty());
    }

    // ── Capacity and overflow ─────────────────────────────────────

    #[test]
    fn test_capacity_limit_evicts_oldest() {
        let mut buf = RingBuffer::new(5);
        buf.push(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        assert_eq!(buf.len(), 5);
        let data = buf.drain();
        assert_eq!(data, vec![3.0, 4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_exact_capacity() {
        let mut buf = RingBuffer::new(3);
        buf.push(&[1.0, 2.0, 3.0]);
        assert_eq!(buf.len(), 3);
        let data = buf.drain();
        assert_eq!(data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_push_one_by_one_at_capacity() {
        let mut buf = RingBuffer::new(3);
        buf.push(&[1.0]);
        buf.push(&[2.0]);
        buf.push(&[3.0]);
        buf.push(&[4.0]); // evicts 1.0
        assert_eq!(buf.len(), 3);
        let data = buf.drain();
        assert_eq!(data, vec![2.0, 3.0, 4.0]);
    }

    // ── Peek ──────────────────────────────────────────────────────

    #[test]
    fn test_peek_partial() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let peeked = buf.peek(3);
        assert_eq!(peeked, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_peek_more_than_available() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0]);
        let peeked = buf.peek(100);
        assert_eq!(peeked, vec![1.0, 2.0]);
    }

    #[test]
    fn test_peek_does_not_consume() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0]);
        let _ = buf.peek(2);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_peek_empty_buffer() {
        let buf = RingBuffer::new(10);
        let peeked = buf.peek(5);
        assert!(peeked.is_empty());
    }

    // ── Clear ─────────────────────────────────────────────────────

    #[test]
    fn test_clear() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0]);
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_clear_empty_buffer() {
        let mut buf = RingBuffer::new(10);
        buf.clear();
        assert!(buf.is_empty());
    }

    // ── Edge cases ────────────────────────────────────────────────

    #[test]
    fn test_push_empty_slice() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[]);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_zero_capacity() {
        let mut buf = RingBuffer::new(0);
        buf.push(&[1.0, 2.0]);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_large_data() {
        let mut buf = RingBuffer::new(1000);
        let data: Vec<f32> = (0..5000).map(|i| i as f32).collect();
        buf.push(&data);
        assert_eq!(buf.len(), 1000);
        let drained = buf.drain();
        assert_eq!(drained.len(), 1000);
        // Should contain the last 1000 samples
        assert_eq!(drained[0], 4000.0);
        assert_eq!(drained[999], 4999.0);
    }

    #[test]
    fn test_negative_samples() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[-1.0, -2.0, -3.0]);
        let data = buf.drain();
        assert_eq!(data, vec![-1.0, -2.0, -3.0]);
    }

    #[test]
    fn test_mixed_values() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[0.0, -1.5, 2.5, f32::MIN, f32::MAX]);
        let data = buf.drain();
        assert_eq!(data.len(), 5);
        assert_eq!(data[0], 0.0);
        assert_eq!(data[1], -1.5);
        assert_eq!(data[2], 2.5);
    }

    // ── Take ──────────────────────────────────────────────────────

    #[test]
    fn test_take_exact_amount() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let taken = buf.take(3);
        assert_eq!(taken, vec![1.0, 2.0, 3.0]);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_take_more_than_available() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0]);
        let taken = buf.take(100);
        assert_eq!(taken, vec![1.0, 2.0]);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_take_zero() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0]);
        let taken = buf.take(0);
        assert!(taken.is_empty());
        assert_eq!(buf.len(), 3);
    }

    // ── Sequential operations ─────────────────────────────────────

    #[test]
    fn test_push_drain_push() {
        let mut buf = RingBuffer::new(5);
        buf.push(&[1.0, 2.0]);
        let _ = buf.drain();
        buf.push(&[3.0, 4.0]);
        let data = buf.drain();
        assert_eq!(data, vec![3.0, 4.0]);
    }

    #[test]
    fn test_multiple_drains() {
        let mut buf = RingBuffer::new(10);
        buf.push(&[1.0, 2.0, 3.0]);
        let d1 = buf.drain();
        assert_eq!(d1, vec![1.0, 2.0, 3.0]);
        let d2 = buf.drain();
        assert!(d2.is_empty());
    }
}
