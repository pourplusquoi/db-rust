pub unsafe fn as_i32(data: &[u8]) -> i32 {
  *(&data[0..4] as *const [u8] as *const i32)
}

pub unsafe fn as_i32_mut(data: &mut [u8]) -> &mut i32 {
  &mut *(&mut data[0..4] as *mut [u8] as *mut i32)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn as_i32_test() {
    let mut data = [0, 0, 0, 0, 0, 0, 0, 0];
    unsafe {
      assert_eq!(0, as_i32(&data));
      assert_eq!(0, as_i32(&data[4..]));
      
      *as_i32_mut(&mut data) = 19260817;
      assert_eq!(19260817, as_i32(&data));
      assert_eq!(0, as_i32(&data[4..]));

      *as_i32_mut(&mut data[4..]) = -20200517;
      assert_eq!(19260817, as_i32(&data));
      assert_eq!(-20200517, as_i32(&data[4..]));
    }
  }
}