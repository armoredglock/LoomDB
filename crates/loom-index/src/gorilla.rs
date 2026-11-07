/// Facebook Gorilla Time-Series Compression Algorithm (VLDB 2015)
/// Performs bit-packed delta-of-deltas encoding on timestamps and
/// XOR-based leading/trailing zero bit-packing on IEEE 754 floating-point values.

pub struct BitWriter {
    bytes: Vec<u8>,
    current_byte: u8,
    bit_count: usize,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            current_byte: 0,
            bit_count: 0,
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        if bit {
            self.current_byte |= 1 << (7 - self.bit_count);
        }
        self.bit_count += 1;
        if self.bit_count == 8 {
            self.bytes.push(self.current_byte);
            self.current_byte = 0;
            self.bit_count = 0;
        }
    }

    pub fn write_bits(&mut self, value: u64, num_bits: usize) {
        for i in (0..num_bits).rev() {
            let bit = ((value >> i) & 1) == 1;
            self.write_bit(bit);
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        if self.bit_count > 0 {
            self.bytes.push(self.current_byte);
        }
        self.bytes
    }
}

pub struct BitReader<'a> {
    bytes: &'a [u8],
    byte_idx: usize,
    bit_idx: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    pub fn read_bit(&mut self) -> Option<bool> {
        if self.byte_idx >= self.bytes.len() {
            return None;
        }
        let bit = (self.bytes[self.byte_idx] >> (7 - self.bit_idx)) & 1;
        self.bit_idx += 1;
        if self.bit_idx == 8 {
            self.byte_idx += 1;
            self.bit_idx = 0;
        }
        Some(bit == 1)
    }

    pub fn read_bits(&mut self, num_bits: usize) -> Option<u64> {
        let mut value = 0u64;
        for _ in 0..num_bits {
            let bit = self.read_bit()?;
            value = (value << 1) | (bit as u64);
        }
        Some(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesPoint {
    pub timestamp_us: u64,
    pub value: f64,
}

pub struct GorillaEncoder {
    writer: BitWriter,
    count: usize,
    prev_timestamp: u64,
    prev_delta: i64,
    prev_value_bits: u64,
    prev_leading: u8,
    prev_trailing: u8,
}

impl GorillaEncoder {
    pub fn new() -> Self {
        Self {
            writer: BitWriter::new(),
            count: 0,
            prev_timestamp: 0,
            prev_delta: 0,
            prev_value_bits: 0,
            prev_leading: u8::MAX,
            prev_trailing: 0,
        }
    }

    pub fn compress_point(&mut self, point: TimeSeriesPoint) {
        if self.count == 0 {
            // First point: store full 64-bit timestamp and 64-bit float
            self.prev_timestamp = point.timestamp_us;
            self.prev_value_bits = point.value.to_bits();
            self.writer.write_bits(self.prev_timestamp, 64);
            self.writer.write_bits(self.prev_value_bits, 64);
            self.count = 1;
            return;
        }

        if self.count == 1 {
            // Second point: store initial delta
            let delta = (point.timestamp_us as i64) - (self.prev_timestamp as i64);
            self.prev_delta = delta;
            self.prev_timestamp = point.timestamp_us;
            self.writer.write_bits(delta as u64, 32);
        } else {
            // Successive points: Delta-of-deltas encoding
            let cur_delta = (point.timestamp_us as i64) - (self.prev_timestamp as i64);
            let dod = cur_delta - self.prev_delta;
            self.prev_delta = cur_delta;
            self.prev_timestamp = point.timestamp_us;

            if dod == 0 {
                self.writer.write_bit(false); // '0'
            } else if dod >= -63 && dod <= 64 {
                self.writer.write_bits(0b10, 2);
                self.writer.write_bits((dod + 63) as u64, 7);
            } else if dod >= -255 && dod <= 256 {
                self.writer.write_bits(0b110, 3);
                self.writer.write_bits((dod + 255) as u64, 9);
            } else if dod >= -2047 && dod <= 2048 {
                self.writer.write_bits(0b1110, 4);
                self.writer.write_bits((dod + 2047) as u64, 12);
            } else {
                self.writer.write_bits(0b1111, 4);
                self.writer.write_bits(dod as u64, 32);
            }
        }

        // Float value XOR compression
        let cur_val_bits = point.value.to_bits();
        let xor = cur_val_bits ^ self.prev_value_bits;
        self.prev_value_bits = cur_val_bits;

        if xor == 0 {
            self.writer.write_bit(false); // Value unchanged: '0'
        } else {
            self.writer.write_bit(true); // Value changed: '1'
            let leading = xor.leading_zeros() as u8;
            let trailing = xor.trailing_zeros() as u8;

            if self.prev_leading != u8::MAX && leading >= self.prev_leading && trailing >= self.prev_trailing {
                // Control bit '0': reuse previous leading/trailing bounds
                self.writer.write_bit(false);
                let significant_bits = 64 - self.prev_leading - self.prev_trailing;
                let payload = xor >> self.prev_trailing;
                self.writer.write_bits(payload, significant_bits as usize);
            } else {
                // Control bit '1': declare new leading and length bounds
                self.writer.write_bit(true);
                let length = 64 - leading - trailing;
                self.writer.write_bits(leading as u64, 6);
                self.writer.write_bits(length as u64, 6);
                let payload = xor >> trailing;
                self.writer.write_bits(payload, length as usize);
                self.prev_leading = leading;
                self.prev_trailing = trailing;
            }
        }

        self.count += 1;
    }

    pub fn finish(self) -> (Vec<u8>, usize) {
        let count = self.count;
        (self.writer.finish(), count)
    }
}

pub struct GorillaDecoder<'a> {
    reader: BitReader<'a>,
    total_count: usize,
    read_count: usize,
    prev_timestamp: u64,
    prev_delta: i64,
    prev_value_bits: u64,
    prev_leading: u8,
    prev_trailing: u8,
}

impl<'a> GorillaDecoder<'a> {
    pub fn new(bytes: &'a [u8], total_count: usize) -> Self {
        Self {
            reader: BitReader::new(bytes),
            total_count,
            read_count: 0,
            prev_timestamp: 0,
            prev_delta: 0,
            prev_value_bits: 0,
            prev_leading: u8::MAX,
            prev_trailing: 0,
        }
    }

    pub fn decompress_all(mut self) -> Option<Vec<TimeSeriesPoint>> {
        let mut points = Vec::with_capacity(self.total_count);
        while self.read_count < self.total_count {
            if self.read_count == 0 {
                let ts = self.reader.read_bits(64)?;
                let val_bits = self.reader.read_bits(64)?;
                self.prev_timestamp = ts;
                self.prev_value_bits = val_bits;
                points.push(TimeSeriesPoint {
                    timestamp_us: ts,
                    value: f64::from_bits(val_bits),
                });
                self.read_count = 1;
                continue;
            }

            if self.read_count == 1 {
                let delta = self.reader.read_bits(32)? as i64;
                self.prev_delta = delta;
                self.prev_timestamp = (self.prev_timestamp as i64 + delta) as u64;
            } else {
                let bit = self.reader.read_bit()?;
                let dod: i64 = if !bit {
                    0
                } else {
                    let b2 = self.reader.read_bit()?;
                    if !b2 {
                        // '10' -> 7 bits
                        let bits = self.reader.read_bits(7)? as i64;
                        bits - 63
                    } else {
                        let b3 = self.reader.read_bit()?;
                        if !b3 {
                            // '110' -> 9 bits
                            let bits = self.reader.read_bits(9)? as i64;
                            bits - 255
                        } else {
                            let b4 = self.reader.read_bit()?;
                            if !b4 {
                                // '1110' -> 12 bits
                                let bits = self.reader.read_bits(12)? as i64;
                                bits - 2047
                            } else {
                                // '1111' -> 32 bits
                                self.reader.read_bits(32)? as i32 as i64
                            }
                        }
                    }
                };
                self.prev_delta += dod;
                self.prev_timestamp = (self.prev_timestamp as i64 + self.prev_delta) as u64;
            }

            // Decode float value
            let val_bit = self.reader.read_bit()?;
            let val_bits = if !val_bit {
                self.prev_value_bits
            } else {
                let control_bit = self.reader.read_bit()?;
                let xor = if !control_bit {
                    let sig_bits = 64 - self.prev_leading - self.prev_trailing;
                    let payload = self.reader.read_bits(sig_bits as usize)?;
                    payload << self.prev_trailing
                } else {
                    let leading = self.reader.read_bits(6)? as u8;
                    let length = self.reader.read_bits(6)? as u8;
                    let trailing = 64 - leading - length;
                    let payload = self.reader.read_bits(length as usize)?;
                    self.prev_leading = leading;
                    self.prev_trailing = trailing;
                    payload << trailing
                };
                self.prev_value_bits ^ xor
            };

            self.prev_value_bits = val_bits;
            points.push(TimeSeriesPoint {
                timestamp_us: self.prev_timestamp,
                value: f64::from_bits(val_bits),
            });
            self.read_count += 1;
        }

        Some(points)
    }
}

/// Compute compression savings ratio as a percentage.
pub fn compression_ratio(raw_bytes: usize, compressed_bytes: usize) -> f64 {
    if raw_bytes == 0 {
        return 0.0;
    }
    ((raw_bytes.saturating_sub(compressed_bytes)) as f64 / raw_bytes as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gorilla_compression_roundtrip_and_ratio() {
        let mut encoder = GorillaEncoder::new();
        let mut original = Vec::new();

        let base_ts = 1_700_000_000_000_000u64;
        let mut cur_val = 24.5f64;

        for i in 0..100 {
            // Predictable 1-second interval + minor floating fluctuations
            let ts = base_ts + i * 1_000_000;
            if i % 5 == 0 {
                cur_val += 0.2;
            }
            let pt = TimeSeriesPoint { timestamp_us: ts, value: cur_val };
            original.push(pt);
            encoder.compress_point(pt);
        }

        let raw_bytes = original.len() * 16; // 8 bytes ts + 8 bytes float
        let (compressed, count) = encoder.finish();
        assert_eq!(count, 100);

        let ratio = compression_ratio(raw_bytes, compressed.len());
        // Gorilla easily delivers >75% compression on typical time-series
        assert!(ratio > 70.0, "Expected >70% compression, got {:.2}%", ratio);

        let decoder = GorillaDecoder::new(&compressed, count);
        let recovered = decoder.decompress_all().unwrap();
        assert_eq!(recovered.len(), original.len());

        for (orig, rec) in original.iter().zip(recovered.iter()) {
            assert_eq!(orig.timestamp_us, rec.timestamp_us);
            assert!((orig.value - rec.value).abs() < 1e-12);
        }
    }
}
