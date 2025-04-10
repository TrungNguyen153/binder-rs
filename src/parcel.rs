use std::{
    ffi::c_void,
    io::{Cursor, Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::BinderResult;

#[derive(Default)]
pub struct Parcel {
    cursor: Cursor<Vec<u8>>,
    object_offsets: Vec<usize>,
    objects_position: usize,
}

impl AsRef<[u8]> for Parcel {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for Parcel {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl Parcel {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cursor: Cursor::new(vec![0; capacity]),
            ..Default::default()
        }
    }

    pub fn from_slice(data: impl AsRef<[u8]>) -> Self {
        Self {
            cursor: Cursor::new(data.as_ref().to_vec()),
            ..Default::default()
        }
    }

    pub unsafe fn from_data_and_offsets(
        data: *const u8,
        data_size: usize,
        offset: *const usize,
        offset_size: usize,
    ) -> Self {
        Self {
            cursor: Cursor::new(std::slice::from_raw_parts(data, data_size).to_vec()),
            object_offsets: std::slice::from_raw_parts(offset, offset_size).to_vec(),
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.cursor.set_position(0);
        self.cursor.get_mut().clear();
        self.object_offsets.clear();
        self.objects_position = 0;
    }

    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    pub fn as_slice(&self) -> &[u8] {
        self.cursor.get_ref()
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.cursor.get_mut()
    }

    pub fn len(&self) -> usize {
        self.cursor.get_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.cursor.get_ref().is_empty()
    }

    pub fn offsets_len(&self) -> usize {
        self.object_offsets.len()
    }

    pub fn offsets(&self) -> &[usize] {
        &self.object_offsets
    }

    pub fn offsets_mut(&mut self) -> &mut Vec<usize> {
        &mut self.object_offsets
    }

    pub fn push_object(&mut self) {
        self.object_offsets.push(self.position());
    }

    pub fn has_unread_data(&self) -> bool {
        self.position() != self.len()
    }

    /// Write an i32 to the parcel
    pub fn write_i32(&mut self, data: i32) -> BinderResult<()> {
        self.cursor.write_i32::<LittleEndian>(data)?;
        Ok(())
    }
    /// Write an u32 to the parcel
    pub fn write_u32(&mut self, data: u32) -> BinderResult<()> {
        self.cursor.write_u32::<LittleEndian>(data)?;
        Ok(())
    }
    /// Write an u64 to the parcel
    pub fn write_u64(&mut self, data: u64) -> BinderResult<()> {
        self.cursor.write_u64::<LittleEndian>(data)?;
        Ok(())
    }
    /// Write an u16 to the parcel
    pub fn write_u16(&mut self, data: u16) -> BinderResult<()> {
        self.cursor.write_u16::<LittleEndian>(data)?;
        Ok(())
    }

    /// Write a bool to the parcel
    pub fn write_bool(&mut self, data: bool) -> BinderResult<()> {
        self.write_u32(data as u32)?;
        Ok(())
    }

    /// Write an u8 to the parcel
    pub fn write_u8(&mut self, data: u8) -> BinderResult<()> {
        self.cursor.write_u8(data as u8)?;
        Ok(())
    }

    /// Write an usize to the parcel
    pub fn write_usize(&mut self, data: usize) -> BinderResult<()> {
        self.cursor.write_u64::<LittleEndian>(data as u64)?;
        Ok(())
    }

    /// Write a slice of data to the parcel
    pub fn write(&mut self, data: &[u8]) -> BinderResult<()> {
        let padded_len = (data.len() + 3) & !3;

        let mut data = data.to_vec();
        if padded_len > data.len() {
            data.resize(padded_len, 0);
        };

        self.cursor.write(data.as_slice())?;

        Ok(())
    }

    /// Read an u8 from the parcel
    pub fn read_u8(&mut self) -> BinderResult<u8> {
        Ok(self.cursor.read_u8()?)
    }

    /// Read an u16 from the parcel
    pub fn read_u16(&mut self) -> BinderResult<u16> {
        Ok(self.cursor.read_u16::<LittleEndian>()?)
    }

    /// Read an u32 from the parcel
    pub fn read_u32(&mut self) -> BinderResult<u32> {
        Ok(self.cursor.read_u32::<LittleEndian>()?)
    }

    /// Read an u64 from the parcel
    pub fn read_u64(&mut self) -> BinderResult<u64> {
        Ok(self.cursor.read_u64::<LittleEndian>()?)
    }

    /// Read an usize from the parcel
    pub fn read_usize(&mut self) -> BinderResult<usize> {
        if size_of::<usize>() == size_of::<u32>() {
            Ok(self.read_u32()? as usize)
        } else {
            Ok(self.read_u64()? as usize)
        }
    }

    /// Read an i32 from the parcel
    pub fn read_i32(&mut self) -> BinderResult<i32> {
        Ok(self.cursor.read_i32::<LittleEndian>()?)
    }

    /// Read a void pointer from the parcel
    pub fn read_pointer(&mut self) -> BinderResult<*const c_void> {
        Ok(self.read_usize()? as *const c_void)
    }

    /// Read a slice of size bytes from the parcel
    pub fn read(&mut self, size: usize) -> BinderResult<Vec<u8>> {
        let size = if (size % 4) != 0 {
            size + 4 - (size % 4)
        } else {
            size
        };
        let mut data = vec![0u8; size];
        self.cursor.read(&mut data)?;
        Ok(data)
    }

    /// Read a slice of size bytes from the parcel
    pub fn read_without_alignment(&mut self, size: usize) -> BinderResult<Vec<u8>> {
        let mut data = vec![0u8; size];
        self.cursor.read(&mut data)?;
        Ok(data)
    }

    /// Read an object of type T from the parcel
    pub fn read_object<T>(&mut self) -> BinderResult<T> {
        unsafe {
            let data = std::slice::from_raw_parts(
                self.cursor
                    .get_ref()
                    .as_ptr()
                    .offset(self.cursor.position() as isize),
                size_of::<T>(),
            );
            self.cursor
                .set_position(self.cursor.position() + size_of::<T>() as u64);
            Ok((data.as_ptr() as *const T).read())
        }
    }

    pub fn write_object<T>(&mut self, object: T) -> BinderResult<()> {
        self.object_offsets.push(self.cursor.position() as usize);
        self.cursor.write(unsafe {
            std::slice::from_raw_parts(&object as *const _ as *const u8, size_of::<T>())
        })?;
        Ok(())
    }

    /// Write a string to the parcel
    pub fn write_str16(&mut self, string: &str) -> BinderResult<()> {
        let mut s16: Vec<u8> = vec![];
        self.write_i32(string.len() as i32)?;
        for c in string.encode_utf16() {
            s16.write_u16::<LittleEndian>(c)?;
        }
        s16.write_u16::<LittleEndian>(0)?;

        if s16.len() % 4 != 0 {
            s16.resize(s16.len() + 4 - (s16.len() % 4), 0);
        }

        self.cursor.write_all(s16.as_slice())?;

        Ok(())
    }

    /// Write a string to the parcel
    pub fn write_str(&mut self, string: &str) -> BinderResult<()> {
        let mut s8: Vec<u8> = Vec::with_capacity(string.len() + 1);
        self.write_i32(string.len() as i32)?;
        for c in string.bytes() {
            s8.push(c);
        }
        s8.push(0);

        if s8.len() % 4 != 0 {
            s8.resize(s8.len() + 4 - (s8.len() % 4), 0);
        }

        self.cursor.write_all(s8.as_slice())?;

        Ok(())
    }
}
