pub unsafe fn read_i32(data: &[u8]) -> i32 {
  *(&data[0..4] as *const [u8] as *const i32)
}

pub unsafe fn write_i32(data: &mut [u8], num: i32) {
  *(&mut data[0..4] as *mut [u8] as *mut i32) = num;
}

pub unsafe fn read_str(data: &[u8]) -> &str {
  let mut len = 0;
  for v in data.iter() {
    if *v == 0 {
      break;
    }
    len += 1;
  }
  &*(&data[0..len] as *const [u8] as *const str)
}

pub unsafe fn write_str(data: &mut [u8], name: &str) {
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
  fn read_write_i32() {
    let mut data = [0; 8];
    unsafe {
      assert_eq!(0, read_i32(&data));
      assert_eq!(0, read_i32(&data[4..]));

      write_i32(&mut data, 19260817);
      assert_eq!(19260817, read_i32(&data));
      assert_eq!(0, read_i32(&data[4..]));

      write_i32(&mut data[4..], -20200517);
      assert_eq!(19260817, read_i32(&data));
      assert_eq!(-20200517, read_i32(&data[4..]));
    }
  }

  #[test]
  fn read_write_str() {
    let mut data = [0; 64];
    unsafe {
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
  }

  #[test]
  fn read_write_mixed() {
    let mut data = [0; 4 + (32 + 4) + (32 + 4) + (32 + 4)];
    unsafe {
      write_i32(&mut data[0..], 3);
      write_str(&mut data[4..], "Table A");
      write_i32(&mut data[36..], 19260817);
      write_str(&mut data[40..], "Table B");
      write_i32(&mut data[72..], 20200517);
      write_str(&mut data[76..], "Table C");

      assert_eq!(3, read_i32(&data[0..]));
      assert_eq!("Table A", read_str(&data[4..]));
      assert_eq!(19260817, read_i32(&data[36..]));
      assert_eq!("Table B", read_str(&data[40..]));
      assert_eq!(20200517, read_i32(&data[72..]));
      assert_eq!("Table C", read_str(&data[76..]));
    }
  }
}