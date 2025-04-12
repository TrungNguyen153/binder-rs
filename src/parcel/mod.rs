use parcelable::{Deserialize, Serialize};
use pretty_hex::pretty_hex;

use crate::{
    binder::{binder_type::BinderType, constant::INTERFACE_HEADER, flat_object::BinderFlatObject},
    error::{BinderError, Result},
};
pub mod parcelable;
const STRICT_MODE_PENALTY_GATHER: i32 = 1 << 31;
#[inline]
pub(crate) fn pad_size(len: usize) -> usize {
    (len + 3) & (!3)
}

pub(crate) trait CharType: Clone {
    type Output;
    fn as_i32(&self) -> i32;
    fn from(v: &i32) -> Self::Output;
}

impl CharType for i16 {
    type Output = i16;
    fn as_i32(&self) -> i32 {
        *self as _
    }
    fn from(v: &i32) -> Self::Output {
        *v as _
    }
}

impl CharType for u16 {
    type Output = u16;
    fn as_i32(&self) -> i32 {
        *self as _
    }
    fn from(v: &i32) -> Self::Output {
        *v as _
    }
}

pub(crate) enum ParcelData<T: Clone + Default + 'static> {
    Vec(Vec<T>),
    Slice(&'static mut [T]),
}

impl<T: Clone + Default> ParcelData<T> {
    fn new() -> Self {
        ParcelData::Vec(Vec::new())
    }

    fn with_capacity(capacity: usize) -> Self {
        ParcelData::Vec(Vec::with_capacity(capacity))
    }

    fn from_vec(data: Vec<T>) -> Self {
        ParcelData::Vec(data)
    }

    fn from_raw_parts_mut(data: *mut T, len: usize) -> Self {
        ParcelData::Slice(unsafe { std::slice::from_raw_parts_mut(data, len) })
    }

    fn as_slice(&self) -> &[T] {
        match self {
            ParcelData::Vec(v) => v.as_slice(),
            ParcelData::Slice(s) => s,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            ParcelData::Vec(v) => v.as_mut_slice(),
            ParcelData::Slice(_) => panic!("ParcelData::Slice can't support as_mut_slice()."),
        }
    }

    pub(crate) fn as_ptr(&self) -> *const T {
        match self {
            ParcelData::Vec(v) => v.as_ptr(),
            ParcelData::Slice(s) => s.as_ptr(),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        match self {
            ParcelData::Vec(v) => v.as_mut_ptr(),
            ParcelData::Slice(s) => s.as_mut_ptr(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn set_len(&mut self, len: usize) {
        match self {
            ParcelData::Vec(v) => unsafe { v.set_len(len) },
            _ => panic!("&[u8] can't support set_len()."),
        }
    }

    fn resize(&mut self, len: usize) {
        match self {
            ParcelData::Vec(v) => v.resize_with(len, Default::default),
            _ => panic!("&[u8] can't support resize()."),
        }
    }

    fn capacity(&self) -> usize {
        match self {
            ParcelData::Vec(v) => v.capacity(),
            ParcelData::Slice(s) => s.len(),
        }
    }

    fn reserve(&mut self, additional: usize) {
        match self {
            ParcelData::Vec(v) => v.reserve(additional),
            _ => panic!("&[u8] can't support reserve()."),
        }
    }

    fn push(&mut self, other: T) {
        match self {
            ParcelData::Vec(v) => v.push(other),
            _ => panic!("extend_from_slice() is only available for ParcelData::Vec."),
        }
    }
}

pub type FnFreeBuffer = fn(Option<&Parcel>, usize, usize, usize, usize) -> Result<()>;

/// Parcel converts data into a byte stream (serialization), making it transferable.
/// The receiving side then transforms this byte stream back into its original data form (deserialization).
pub struct Parcel {
    data: ParcelData<u8>,
    pub(crate) objects: ParcelData<usize>,
    pos: usize,
    next_object_hint: usize,
    request_header_present: bool,
    work_source_request_header_pos: usize,
    free_buffer: Option<FnFreeBuffer>,
}

impl Default for Parcel {
    fn default() -> Self {
        Parcel::with_capacity(256)
    }
}

impl Parcel {
    pub fn new() -> Self {
        Parcel::with_capacity(256)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Parcel {
            data: ParcelData::with_capacity(capacity),
            objects: ParcelData::new(),
            pos: 0,
            next_object_hint: 0,
            request_header_present: false,
            work_source_request_header_pos: 0,
            free_buffer: None,
        }
    }

    // We leaking free buffer for now
    // TODO: fix free buffer
    pub fn from_ipc_parts(
        data: *mut u8,
        length: usize,
        objects: *mut usize,
        object_count: usize,
        free_buffer: Option<FnFreeBuffer>,
    ) -> Self {
        Parcel {
            data: ParcelData::from_raw_parts_mut(data, length),
            objects: ParcelData::from_raw_parts_mut(objects, object_count),
            pos: 0,
            next_object_hint: 0,
            request_header_present: false,
            work_source_request_header_pos: 0,
            free_buffer,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Parcel {
            data: ParcelData::from_vec(data),
            objects: ParcelData::new(),
            pos: 0,
            next_object_hint: 0,
            // objects: ptr::null_mut(),
            // object_count: 0,
            request_header_present: false,
            work_source_request_header_pos: 0,
            free_buffer: None,
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn set_data_size(&mut self, new_len: usize) {
        self.data.set_len(new_len);
        if new_len < self.pos {
            self.pos = new_len;
        }
    }

    pub fn close_file_descriptors(&self) {
        for offset in self.objects.as_slice() {
            let obj: &BinderFlatObject =
                unsafe { BinderFlatObject::ref_from_raw(self.data.as_ptr(), *offset) };
            if obj.header_type() == BinderType::Fd {
                // Close the file descriptor
                obj.owned_fd();
            }
        }
    }

    pub fn set_data_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn data_position(&self) -> usize {
        self.pos
    }

    pub fn data_size(&self) -> usize {
        if self.data.len() > self.pos {
            self.data.len()
        } else {
            self.pos
        }
    }

    pub fn has_unread_data(&self) -> bool {
        self.data_position() != self.data_size()
    }

    pub fn unread_data_size(&self) -> usize {
        if self.has_unread_data() {
            return self.data_size() - self.data_position();
        }
        0
    }

    /// Read a type that implements [`Deserialize`] from the sub-parcel.
    pub fn read<D: Deserialize>(&mut self) -> Result<D> {
        D::deserialize(self)
    }

    /// Attempt to read a type that implements [`Deserialize`] from this parcel
    /// onto an existing value. This operation will overwrite the old value
    /// partially or completely, depending on how much data is available.
    pub fn read_onto<D: Deserialize>(&mut self, x: &mut D) -> Result<()> {
        x.deserialize_from(self)
    }

    pub fn data_avail(&self) -> usize {
        let result = self.data.len() - self.pos;
        assert!(result < i32::MAX as _, "data too big: {}", result);

        result
    }

    pub(crate) fn read_aligned_data(&mut self, len: usize) -> Result<&[u8]> {
        let aligned = pad_size(len);
        let pos = self.pos;

        if aligned <= self.data_avail() {
            self.pos = pos + aligned;
            Ok(&self.data.as_slice()[pos..pos + len])
        } else {
            error!(
                "Not enough data to read aligned data.: {aligned} <= {}",
                self.data_avail()
            );
            Err(BinderError::NotEnoughData)
        }
    }

    pub(crate) fn read_object(&mut self, null_meta: bool) -> Result<&BinderFlatObject> {
        let data_pos = self.pos;
        let size = std::mem::size_of::<BinderFlatObject>();

        let obj: &BinderFlatObject =
            unsafe { BinderFlatObject::ref_from_raw(self.read_aligned_data(size)?.as_ptr(), 0) };

        if !null_meta && obj.cookie() == 0 && obj.pointer() == 0 {
            return Ok(obj);
        }

        let objects = self.objects.as_slice();
        let count = objects.len();
        let mut opos = self.next_object_hint;

        if count > 0 {
            trace!("Parcel looking for obj at {}, hint={}", data_pos, opos);
            if opos < count {
                while opos < (count - 1) && objects[opos] < data_pos {
                    opos += 1;
                }
            } else {
                opos = count - 1;
            }
            if objects[opos] == data_pos {
                self.next_object_hint = opos + 1;
                return Ok(obj);
            }

            while opos > 0 && objects[opos] > data_pos {
                opos -= 1;
            }

            if objects[opos] == data_pos {
                self.next_object_hint = opos + 1;
                return Ok(obj);
            }
        }
        error!("Parcel: unable to find object at index {}", data_pos);
        Err(BinderError::BadType)
    }

    /// Safely read a sized parcelable.
    ///
    /// Read the size of a parcelable, compute the end position
    /// of that parcelable, then build a sized readable sub-parcel
    /// and call a closure with the sub-parcel as its parameter.
    /// The closure can keep reading data from the sub-parcel
    /// until it runs out of input data.
    /// After the closure returns, skip to the end of the current
    /// parcelable regardless of how much the closure has read.
    ///
    pub fn sized_read<F>(&mut self, f: F) -> Result<()>
    where
        for<'b> F: FnOnce(&mut Parcel) -> Result<()>,
    {
        let start = self.data_position();
        let parcelable_size: i32 = self.read()?;
        if parcelable_size < 4 {
            error!("Parcel: bad size for object: {}", parcelable_size);
            return Err(BinderError::BadValue);
        }

        let end = start.checked_add(parcelable_size as _).ok_or_else(|| {
            error!("Parcel: check_add error: {}", parcelable_size);
            BinderError::BadValue
        })?;
        if end > self.data_size() {
            error!("Parcel: not enough data: {} > {}", end, self.data_size());
            return Err(BinderError::NotEnoughData);
        }

        f(self)?;

        // Advance the data position to the actual end,
        // in case the closure read less data than was available
        self.set_data_position(end);

        Ok(())
    }

    pub(crate) fn read_array<D: Deserialize + ?Sized>(&mut self) -> Result<Option<Vec<D>>> {
        let len: i32 = self.read()?;
        if len < -1 {
            error!("Parcel: bad array length: {}", len);
            return Err(BinderError::BadValue);
        }
        if len <= 0 {
            return Ok(None);
        }

        let size = len as usize * std::mem::size_of::<D>();
        let padded = pad_size(size);

        if padded > self.data_avail() {
            error!(
                "Parcel: not enough data to read array: {} > {}",
                padded,
                self.data_avail()
            );
            return Err(BinderError::NotEnoughData);
        }

        let mut result = Vec::with_capacity(len as usize);
        let pos = self.pos;
        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                self.data.as_slice()[pos..pos + size].as_ptr(),
                result.as_mut_ptr() as _,
                size,
            );
            result.set_len(len as usize);
        }
        self.set_data_position(pos + padded);

        Ok(Some(result))
    }

    pub(crate) fn read_array_char<D: CharType>(
        &mut self,
    ) -> Result<Option<Vec<<D as CharType>::Output>>> {
        let len: i32 = self.read()?;
        if len < -1 {
            error!("Parcel: bad array length: {}", len);
            return Err(BinderError::BadValue);
        }
        if len <= 0 {
            return Ok(None);
        }

        let size = len as usize * 4;
        let padded = pad_size(size);

        if padded > self.data_avail() {
            error!(
                "Parcel: not enough data to read array char: {} > {}",
                padded,
                self.data_avail()
            );
            return Err(BinderError::NotEnoughData);
        }

        let mut result = Vec::with_capacity(len as usize);
        let pos = self.pos;
        let (_, ints, _) = unsafe { self.data.as_slice()[pos..pos + size].align_to::<i32>() };
        for i in ints {
            result.push(D::from(i));
        }

        self.set_data_position(pos + padded);

        Ok(Some(result))
    }

    /// Read a vector size from the parcel and resize the given output vector to
    /// be correctly sized for that amount of data.
    ///
    /// This method is used in AIDL-generated server side code for methods that
    /// take a mutable slice reference parameter.
    pub fn resize_out_vec<D: Default + Deserialize>(&mut self, out_vec: &mut Vec<D>) -> Result<()> {
        let len: i32 = self.read()?;

        if len < 0 {
            return Err(BinderError::UnexpectedNull);
        }

        // usize in Rust may be 16-bit, so i32 may not fit
        let len = len.try_into().or(Err(BinderError::BadValue))?;
        out_vec.resize_with(len, Default::default);

        Ok(())
    }

    /// Read a vector size from the parcel and either create a correctly sized
    /// vector for that amount of data or set the output parameter to None if
    /// the vector should be null.
    ///
    /// This method is used in AIDL-generated server side code for methods that
    /// take a mutable slice reference parameter.
    pub fn resize_nullable_out_vec<D: Default + Deserialize>(
        &mut self,
        out_vec: &mut Option<Vec<D>>,
    ) -> Result<()> {
        let len: i32 = self.read()?;

        if len < 0 {
            *out_vec = None;
        } else {
            // usize in Rust may be 16-bit, so i32 may not fit
            let len = len.try_into().or(Err(BinderError::BadValue))?;
            let mut vec = Vec::with_capacity(len);
            vec.resize_with(len, Default::default);
            *out_vec = Some(vec);
        }

        Ok(())
    }

    pub(crate) fn update_work_source_request_header_pos(&mut self) {
        if !self.request_header_present {
            self.work_source_request_header_pos = self.data.len();
            self.request_header_present = true;
        }
    }

    pub fn write<S: Serialize + ?Sized>(&mut self, parcelable: &S) -> Result<()> {
        parcelable.serialize(self)
    }

    pub(crate) fn write_array<S: Serialize + Sized>(&mut self, parcelable: &[S]) -> Result<()> {
        let len = parcelable.len();
        self.write::<i32>(&(len as _))?;

        if len == 0 {
            return Ok(());
        }

        let size = std::mem::size_of_val(parcelable);
        let padded = pad_size(size);
        let pos = self.pos;

        self.data.reserve(pos + padded);
        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                parcelable.as_ptr() as _,
                self.data.as_mut_ptr().add(pos) as _,
                size,
            );
            if self.data.len() < pos + padded {
                self.data.set_len(pos + padded);
            }
        }

        self.set_data_position(pos + padded);

        Ok(())
    }

    pub(crate) fn write_array_char<S: CharType>(&mut self, parcelable: &[S]) -> Result<()> {
        let len = parcelable.len();
        self.write::<i32>(&(len as _))?;

        let size = 4 * len;
        let padded = pad_size(size);

        self.data.reserve(self.pos + padded);
        for c in parcelable {
            self.write(&c.as_i32())?;
        }

        Ok(())
    }

    /// Writes the length of a slice to the parcel.
    ///
    /// This is used in AIDL-generated client side code to indicate the
    /// allocated space for an output array parameter.
    pub fn write_slice_size<T>(&mut self, slice: Option<&[T]>) -> Result<()> {
        if let Some(slice) = slice {
            let len: i32 = slice.len().try_into().or(Err(BinderError::BadValue))?;
            self.write(&len)
        } else {
            self.write(&-1i32)
        }
    }

    pub(crate) fn write_aligned<T>(&mut self, val: &T) {
        let unaligned = std::mem::size_of::<T>();
        let val_bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(val as *const T as *const u8, unaligned) };

        self.write_aligned_data(val_bytes);
    }

    pub(crate) fn write_aligned_data(&mut self, data: &[u8]) {
        let unaligned = data.len();
        let aligned = pad_size(unaligned);
        let pos = self.pos;
        info!("pos={pos} unaligned={unaligned} - aligned={aligned}");

        self.data.reserve(pos + aligned);

        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                data.as_ptr() as _,
                self.data.as_mut_ptr().add(pos) as _,
                unaligned,
            );
            if pos + aligned > self.data.len() {
                self.data.set_len(pos + aligned);
            }
        }

        self.set_data_position(pos + aligned);
    }

    pub(crate) fn write_object(&mut self, obj: &BinderFlatObject, null_meta: bool) -> Result<()> {
        let data_pos = self.pos;
        self.write_aligned(obj);

        if null_meta || obj.pointer() != 0 {
            obj.acquire()?;
            self.objects.push(data_pos as _);
        }

        Ok(())
    }

    pub(crate) fn write_interface_token(&mut self, interface: &str) -> Result<()> {
        // strict mode policy: 0x42000004
        // this hardcode for fast
        // TODO : implement get strict mode policy
        self.write(&(0x42000004 | STRICT_MODE_PENALTY_GATHER))?;
        self.update_work_source_request_header_pos();
        // let work_source: i32 = if thread_state::should_propagate_work_source() {
        //     thread_state::calling_work_source_uid() as _
        // } else {
        //     thread_state::UNSET_WORK_SOURCE
        // };
        let work_source: i32 = -1;
        self.write(&work_source)?;
        self.write(&INTERFACE_HEADER)?;
        self.write(&interface)?;

        Ok(())
    }

    /// Perform a series of writes to the parcel, prepended with the length
    /// (in bytes) of the written data.
    ///
    /// The length `0i32` will be written to the parcel first, followed by the
    /// writes performed by the callback. The initial length will then be
    /// updated to the length of all data written by the callback, plus the
    /// size of the length elemement itself (4 bytes).
    ///
    /// # Examples
    ///
    /// After the following call:
    ///
    /// ```
    /// # use rsbinder::{Binder, Interface, Parcel};
    /// # let mut parcel = Parcel::new();
    /// parcel.sized_write(|subparcel| {
    ///     subparcel.write(&1u32)?;
    ///     subparcel.write(&2u32)?;
    ///     subparcel.write(&3u32)
    /// });
    /// ```
    ///
    /// `parcel` will contain the following:
    ///
    /// ```ignore
    /// [16i32, 1u32, 2u32, 3u32]
    /// ```
    pub fn sized_write<F>(&mut self, f: F) -> Result<()>
    where
        for<'b> F: FnOnce(&mut Parcel) -> Result<()>,
    {
        let start = self.data_position();
        self.write(&0i32)?;
        {
            f(self)?;
        }
        let end = self.data_position();
        self.set_data_position(start);
        assert!(end >= start);
        self.write::<i32>(&((end - start) as _))?;
        self.set_data_position(end);
        Ok(())
    }

    pub(crate) fn append_all_from(&mut self, other: &mut Parcel) -> Result<()> {
        self.append_from(other, 0, other.data_size())
    }

    pub(crate) fn append_from(
        &mut self,
        other: &mut Parcel,
        offset: usize,
        size: usize,
    ) -> Result<()> {
        if size == 0 {
            return Ok(());
        }
        if size > i32::MAX as usize {
            error!("Parcel::append_from: the size is too large: {}", size);
            return Err(BinderError::BadValue);
        }
        let other_len = other.data_size();
        if offset > other_len || size > other_len || (offset + size) > other_len {
            error!(
                "Parcel::append_from: The given offset({}) and size({}) exceed the data range of the parcel.",
                offset, size
            );
            return Err(BinderError::BadValue);
        }

        let start_pos = self.pos;
        let mut first_idx: i32 = -1;
        let mut last_idx: i32 = -2;
        {
            let object_size = std::mem::size_of::<BinderFlatObject>();
            let objects = self.objects.as_slice();

            for (i, &off) in objects.iter().enumerate() {
                if off >= offset as _ && (off + object_size) <= (offset + size) {
                    if first_idx == -1 {
                        first_idx = i as i32;
                    }
                    last_idx = i as i32;
                }
            }
        }

        let num_objects = last_idx - first_idx + 1;

        self.data.reserve(self.pos + size);
        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                other.data.as_slice()[offset..offset + size].as_ptr() as _,
                self.data.as_mut_ptr().add(self.pos) as _,
                size,
            );
            if self.pos + size > self.data.len() {
                self.data.set_len(self.pos + size);
            }
        }
        self.set_data_position(self.pos + size);

        if num_objects > 0 {
            let mut idx = self.objects.len();
            self.objects.resize(idx + (num_objects as usize));

            let objects = self.objects.as_mut_slice();
            for i in first_idx..=last_idx {
                let off = objects[i as usize] as usize - offset + start_pos;
                objects[idx] = off as _;
                idx += 1;
                let flat: &mut BinderFlatObject =
                    unsafe { BinderFlatObject::mut_from_raw(self.data.as_mut_ptr(), off) };
                flat.acquire()?;
                if flat.header_type() == BinderType::Fd {
                    //                    flat.set_handle(nix::fcntl::fcntl(flat.handle() as _, nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0))? as _);
                    flat.set_handle(nix::fcntl::fcntl(
                        flat.handle() as _,
                        nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0),
                    )? as _);
                    flat.set_cookie(1);
                }
            }
        }

        Ok(())
    }

    fn release_objects(&self) {
        if self.objects.len() == 0 {
            return;
        }

        for pos in self.objects.as_slice() {
            let obj: &BinderFlatObject =
                unsafe { BinderFlatObject::ref_from_raw(self.data.as_ptr(), *pos as usize) };
            obj.release()
                .map_err(|e| error!("Parcel: unable to release object: {:?}", e))
                .ok();
        }
    }
}

impl Drop for Parcel {
    fn drop(&mut self) {
        match self.free_buffer {
            Some(free_buffer) => {
                free_buffer(
                    Some(self),
                    self.data.as_ptr() as _,
                    self.data.len(),
                    self.objects.as_ptr() as _,
                    self.objects.len(),
                )
                .unwrap();
            }
            None => {
                self.release_objects();
            }
        }
    }
}

impl std::fmt::Debug for Parcel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Parcel: pos {}, len {}", self.pos, self.data.len())?;
        if self.objects.len() > 0 {
            let bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    self.objects.as_ptr() as *const u8,
                    self.objects.len() * std::mem::size_of::<usize>(),
                )
            };
            writeln!(
                f,
                "Object count {}\n{}",
                self.objects.len(),
                pretty_hex(&bytes)
            )?;
        }
        write!(f, "{}", pretty_hex(&self.data.as_slice()))
    }
}

impl<const N: usize> TryFrom<&mut Parcel> for [u8; N] {
    type Error = BinderError;

    fn try_from(parcel: &mut Parcel) -> Result<Self> {
        let data = parcel.read_aligned_data(N)?;
        Ok(<[u8; N] as TryFrom<&[u8]>>::try_from(data)?)
    }
}
