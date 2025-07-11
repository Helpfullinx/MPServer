use bitstream_io::{BigEndian, BitWrite2, BitWriter};
use std::io::Cursor;

const QUANTIZE_SCALE: f32 = 1000.0;

// fn quantize_delta(delta: f32) -> i16 {
//     (delta * QUANTIZE_SCALE).round() as i16
// }
// 
// #[test]
// fn test_encode_position_delta() {
//     let position1 = Vec2::new(0.0, 0.0);
//     let position2 = Vec2::new(8.0, 8.0);
// 
//     let quantized_values = vec![31, 64, 31, 64];
//     let result = encode_position_delta(position1, position2);
// 
//     println!("{:?}", quantized_values);
//     println!("{:?}", result);
//     assert_eq!(quantized_values, result,)
// }
// 
// pub fn encode_position_delta(position_prev: Vec2, position_cur: Vec2) -> Vec<u8> {
//     let dx = quantize_delta(position_cur.x - position_prev.x);
//     let dy = quantize_delta(position_cur.y - position_prev.y);
// 
//     let mut buff = Vec::new();
//     let mut writer = BitWriter::endian(Cursor::new(&mut buff), BigEndian);
// 
//     writer.write(16, dx).unwrap();
//     writer.write(16, dy).unwrap();
// 
//     buff
// }

// pub fn decode_position_delta(position_prev: Position, position_cur: Position) -> Position {}
