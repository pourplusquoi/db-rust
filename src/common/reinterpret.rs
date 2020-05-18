pub fn read_u32(data: &[u8]) -> u32 {
  unsafe {
    *(&data[0..4] as *const [u8] as *const u32)
  }
}

pub fn write_u32(data: &mut [u8], num: u32) {
  unsafe {
    *(&mut data[0..4] as *mut [u8] as *mut u32) = num;
  }
}

pub fn read_u64(data: &[u8]) -> u64 {
  unsafe {
    *(&data[0..8] as *const [u8] as *const u64)
  }
}

pub fn write_u64(data: &mut [u8], num: u64) {
  unsafe {
    *(&mut data[0..8] as *mut [u8] as *mut u64) = num;
  }
}

pub fn read_i32(data: &[u8]) -> i32 {
  unsafe {
    *(&data[0..4] as *const [u8] as *const i32)
  }
}

pub fn write_i32(data: &mut [u8], num: i32) {
  unsafe {
    *(&mut data[0..4] as *mut [u8] as *mut i32) = num;
  }
}

pub fn read_str(data: &[u8]) -> &str {
  let mut len = 0;
  for v in data.iter() {
    if *v == 0 {
      break;
    }
    len += 1;
  }
  unsafe {
    &*(&data[0..len] as *const [u8] as *const str)
  }
}

pub fn write_str(data: &mut [u8], name: &str) {
  for (src, dst) in name.as_bytes().iter().zip(data.iter_mut()) {
    *dst = *src;
  }
  if name.len() < data.len() {
    data[name.len()] = 0;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_write_u32() {
    let mut data = [0; 8];
    assert_eq!(0, read_u32(&data));
    assert_eq!(0, read_u32(&data[4..]));

    write_u32(&mut data, 19260817);
    assert_eq!(19260817, read_u32(&data));
    assert_eq!(0, read_u32(&data[4..]));

    write_u32(&mut data[4..], 20200517);
    assert_eq!(19260817, read_u32(&data));
    assert_eq!(20200517, read_u32(&data[4..]));
  }

  #[test]
  fn read_write_i32() {
    let mut data = [0; 8];
    assert_eq!(0, read_i32(&data));
    assert_eq!(0, read_i32(&data[4..]));

    write_i32(&mut data, 19260817);
    assert_eq!(19260817, read_i32(&data));
    assert_eq!(0, read_i32(&data[4..]));

    write_i32(&mut data[4..], -20200517);
    assert_eq!(19260817, read_i32(&data));
    assert_eq!(-20200517, read_i32(&data[4..]));
  }

  #[test]
  fn read_write_str() {
    let mut data = [0; 64];
    assert_eq!("", read_str(&data));
    assert_eq!("", read_str(&data[32..]));

    write_str(&mut data, "hello world");
    assert_eq!("hello world", read_str(&data));
    assert_eq!("", read_str(&data[32..]));

    let str_32_bytes = "12345678901234567890123456789012";
    write_str(&mut data[32..], str_32_bytes);
    assert_eq!("hello world", read_str(&data));
    assert_eq!(str_32_bytes, read_str(&data[32..]));
  }

  #[test]
  fn read_write_mixed() {
    let mut data = [0; 12 + (32 + 4) + (32 + 4) + (32 + 4)];

    write_u64(&mut data[0..], 18042398900264319379);
    write_u32(&mut data[8..], 3);
    write_str(&mut data[12..], "Table A");
    write_i32(&mut data[44..], 19260817);
    write_str(&mut data[48..], "Table B");
    write_i32(&mut data[80..], 20200517);
    write_str(&mut data[84..], "Table C");
    write_i32(&mut data[116..], -1);

    assert_eq!(18042398900264319379, read_u64(&data[0..]));
    assert_eq!(3, read_u32(&data[8..]));
    assert_eq!("Table A", read_str(&data[12..]));
    assert_eq!(19260817, read_i32(&data[44..]));
    assert_eq!("Table B", read_str(&data[48..]));
    assert_eq!(20200517, read_i32(&data[80..]));
    assert_eq!("Table C", read_str(&data[84..]));
    assert_eq!(-1, read_i32(&data[116..]));
  }
}