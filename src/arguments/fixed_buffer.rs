
// Module includes 2 C style fixed buffer arrays that can be used with u8 and u16
#[allow(dead_code)]
pub mod u16 {
  use std::io::{BufWriter, Write, Error};                                                 
  use std::fs::OpenOptions;
  
  pub const FIXED_BUF_SIZE: usize = 2048;

  #[derive(Copy, Clone, Debug)]
  pub struct U16FixedBuffer {
    pub buffer: [u16; FIXED_BUF_SIZE],  // Arrays have a fixed size of 2048.
    pub len: usize,                     // len tracks the content of the string
    pub cap: usize,                     // Cap stores the max size of the array.
  }


  impl U16FixedBuffer {

    /**Function Creates a new Fixed buffer filled with zeros.
     * Params:
     *  None.
     * Returns U16FixedBuffer.
    */
    pub fn new() -> U16FixedBuffer {
      U16FixedBuffer { buffer: [0u16; FIXED_BUF_SIZE], len: 0, cap: FIXED_BUF_SIZE }
    }
  
    /**Function Pushes a slice u8 slice into the contents of the array.
     * Params:
     *  &self,
     *  slice: &str {The u8 slice you want to push into the array.}
     * Returns usize.
    */
    pub fn push_str(&mut self, slice: &str) -> usize {
      if slice.len().clone() + self.len.clone() > self.cap.clone() {
        return 0 as usize;
      }
  
      let mut pos: usize = self.len.clone();
      for i in slice.chars() {
        self.buffer[pos] = i as u16;
        pos += 1;
      }
  
      self.len = pos;
      
      pos
    }
  
    /**Function pushes a single character into the array.
     * Params:
     *  &self,
     *  ch: char {The char you want to push.}
     * Returns usize.
    */
    pub fn push(&mut self, ch: char) -> usize {
      if self.len.clone()+1 > self.cap.clone() {
        return 0 as usize;
      }
  
      let mut pos: usize = self.len.clone();
      self.buffer[pos] = ch as u16;
      pos += 1;
  
      self.len = pos;
  
      pos
    }
  
    /**Function removes the last character in the array.
     * Params:
     *  &self.
     * Returns nothing.
    */
    pub fn pop(&mut self) -> () {
      self.buffer[self.len] = 0u16;
    }
  
    /**Function clears the buffer and sets the contents to zero. Simmilar to memset in C
     * Params:
     *  &self.
     * Returns nothing.
    */
    pub fn clear(&mut self) -> () {
      let mut counter: usize = self.len.clone();
      
      for i in 0..self.len {
        self.buffer[i] = 0u16;
        counter -= 1;
      }
  
      self.len = counter;
    }
  
    /**Function takes the string content of the buffer and writes it to a file.
     * Params:
     *  &self,
     *  file_name: &str {The name of the file.}
     * Returns Result<usize, Error>
     */
    pub fn write_data(&self, file_name: &str) -> Result<usize, Error> {
      let temp_string = String::from_utf16_lossy(&self.buffer);
      
      match OpenOptions::new().read(true).append(true).open(file_name) {
        Ok(s) => {
          let mut writer = BufWriter::new(s);

          match writer.write(temp_string.as_str().as_bytes()) {
            Ok(s) => { return Ok(s); }
            Err(e) => { return Err(e); }
          }
        },
        Err(e) => { return Err(e); }
      }
    }
  }
}

#[allow(dead_code)]
pub mod u8 {
  use std::io::{BufWriter, Write, Error};
  use std::fs::OpenOptions;

  pub const FIXED_BUF_SIZE: usize = 2048;

  #[derive(Copy, Clone, Debug)]
  pub struct U8FixedBuffer {
    pub buffer: [u8; FIXED_BUF_SIZE],
    pub len: usize,
    pub cap: usize,
  }

  impl U8FixedBuffer {

    /**Function Creates a new Fixed buffer filled with zeros.
     * Params:
     *  None.
     * Returns U8FixedBuffer.
    */
    pub fn new() -> U8FixedBuffer {
      U8FixedBuffer { buffer: [0u8; FIXED_BUF_SIZE], len: 0, cap: FIXED_BUF_SIZE }
    }

    /**Function Pushes a slice u8 slice into the contents of the array.
     * Params:
     *  &self,
     *  slice: &str {The u8 slice you want to push into the array.}
     * Returns usize.
    */
    pub fn push_str(&mut self, slice: &str) -> usize {
      if slice.len().clone() + self.len.clone() > self.cap.clone() {
        return 0 as usize;
      }

      let mut pos: usize = self.len.clone();
      for i in slice.chars() {
        self.buffer[pos] = i as u8;
        pos += 1;
      }

      self.len = pos;
      
      pos
    }

    /**Function pushes a single character into the array.
     * Params:
     *  &self,
     *  ch: char {The char you want to push.}
     * Returns usize.
    */
    pub fn push(&mut self, ch: char) -> usize {
      if self.len.clone()+1 > self.cap.clone() {
        return 0 as usize;
      }

      let mut pos: usize = self.len.clone();
      self.buffer[pos] = ch as u8;
      pos += 1;

      self.len = pos;

      pos
    }

    /**Function removes the last character in the array.
     * Params:
     *  &self.
     * Returns nothing.
    */
    pub fn pop(&mut self) -> () {
      self.buffer[self.len] = 0u8;
    }

    /**Function clears the buffer and sets the contents to zero. Simmilar to memset in C
     * Params:
     *  &self.
     * Returns nothing.
    */
    pub fn clear(&mut self) -> () {
      let mut counter: usize = self.len.clone();
      
      for i in 0..self.len {
        self.buffer[i] = 0u8;
        counter -= 1;
      }

      self.len = counter;
    }

    /**Function takes the string content of the buffer and writes it to a file.
     * Params:
     *  &self,
     *  file_name: &str {The name of the file.}
     * Returns Result<usize, Error>
     */
    pub fn write_data(&self, file_name: &str) -> Result<usize, Error> {
      let buffer = &self.buffer[0..self.len];

      match OpenOptions::new().read(true).append(true).open(file_name) {
        Ok(s) => {
          let mut writer = BufWriter::new(s);

          match writer.write(buffer) {
            Ok(s) => { return Ok(s); }
            Err(e) => { return Err(e); }
          }
        },
        Err(e) => { return Err(e); }
      }
    }
  }
}