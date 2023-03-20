use crate::Pages;
/// A [`Vec`]-like type located in memory pages acquired directly from the kernel. For big lengths a faster to
/// allocate/deallocate than a normal [`Vec`], but considerably slower for small sizes. Intended to be used for very large data
/// sets, with a rough estimate of capacity known ahead of time.
/// # Advantages:
/// 1. 2-3x times faster than default allocator for big vec sizes (over ~20 MB).
/// 2. memory is released directly to the kernel as soon as [`PagedVec`] is dropped, which may not always be the case for
/// standard allocator, leading to decreased memory footprint.
/// 3. More conservative growth. Since [`PagedVec`] is intended for very large sizes, it is considerably more conservative with
/// allocating memory(1.5x previous cap instead of 2x for standard [`Vec`].
/// # Disadvantages
/// 1. Slower for small data sets
/// 2. Can't be turned into a [`Box<[T]>`]
use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
pub struct PagedVec<T: Sized> {
    data: Pages<crate::AllowRead, crate::AllowWrite, crate::DenyExec>,
    len: usize,
    pd: PhantomData<T>,
}
impl<T: Sized> PagedVec<T> {
    /// Creates a new [`PagedVec`] with `capacity`.
    /// # Examples
    /// ```
    /// # use pages::*;
    /// // capacity must be specified!
    /// let mut vec = PagedVec::new(0x1000);
    /// vec.push(0.0);
    /// ```
    pub fn new(capacity: usize) -> Self {
        let bytes_min = (capacity * std::mem::size_of::<T>()).max(0x1000);
        let data = Pages::new(bytes_min);
        Self {
            data,
            len: 0,
            pd: PhantomData,
        }
    }
    /// Pushes `t` into `self` if under capacity, else returns `t`.
    pub fn push_within_capacity(&mut self, t: T) -> Result<(), T> {
        if self.len * std::mem::size_of::<T>() < self.data.len() {
            let slice = unsafe {
                std::slice::from_raw_parts_mut(self.data.get_ptr_mut(0).cast::<T>(), self.len + 1)
            };
            slice[self.len] = t;
            self.len += 1;
            Ok(())
        } else {
            Err(t)
        }
    }
    fn get_next_cap(cap: usize) -> usize {
        (cap + cap / 2).max(0x1000)
    }
    fn resize(&mut self, next_cap: usize) {
        let bytes_cap = next_cap * std::mem::size_of::<T>();
        let mut data = Pages::new(bytes_cap);
        let cpy_len = self.len() * std::mem::size_of::<T>();
        data.split_at_mut(cpy_len)
            .0
            .copy_from_slice(self.data.split_at_mut(cpy_len).0);
        self.data = data;
    }
    /// Reserves capacity for at least additional more elements to be inserted in the given [`PagedVec<T>`]. The collection may
    /// reserve more space to speculatively avoid frequent reallocations. After calling reserve, capacity will be greater than
    /// or equal to self.len() + additional. Does nothing if capacity is already sufficient.
    pub fn reserve(&mut self, additional: usize) {
        if self.len() + additional < self.capacity() {
            return;
        }
        self.resize(Self::get_next_cap(self.len() + additional));
    }
    /// Reserves the minimum capacity for at least additional more elements to be inserted in the given [`PagedVec<T>`]. Unlike
    /// reserve, this will not deliberately over-allocate to speculatively avoid frequent allocations. After calling
    /// [`Self::reserve_exact`], capacity will be greater than or equal to self.len() + additional. Does nothing if the capacity is
    /// already sufficient.
    ///
    ///
    /// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied upon
    /// to be precisely minimal. Using reserve before [`Self::push`] is preferred over using just [`Self::push`], because
    /// reallocation's of [`PagedVec`] are slow.
    pub fn reserve_exact(&mut self, additional: usize) {
        if self.len() + additional < self.capacity() {
            return;
        }
        self.resize(self.len() + additional);
    }
    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// Note: Because this shifts over the remaining elements, it has a
    /// worst-case performance of *O*(*n*).
    pub fn remove(&mut self, index: usize) -> T {
        // Taken form std lib.
        let ret;
        unsafe {
            // the place we are taking from.
            let ptr = self.as_mut_ptr().add(index);
            // copy it out, unsafely having a copy of the value on
            // the stack and in the vector at the same time.
            ret = std::ptr::read(ptr);

            // Shift everything down to fill in that spot.
            std::ptr::copy(ptr.add(1), ptr, self.len - index - 1);
        }
        self.len -= 1;
        ret
    }
    /// Pushes `t` into `self` and reallocates if over capacity. Generally unadvised, because reallocation's of [`PagedVec`]-s
    /// are very slow. Setting sufficient capacity and using [`Self::push_within_capacity`] is generally encouraged.  
    pub fn push(&mut self, t: T) {
        if let Err(t) = self.push_within_capacity(t) {
            self.resize(Self::get_next_cap(self.capacity()));
            match self.push_within_capacity(t) {
                Ok(_) => (),
                Err(_) => panic!("PagedVec expanded, but still had not enough space for a push!"),
            }
        }
    }
    /// Gets the capacity of `self`.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.len() / std::mem::size_of::<T>()
    }
    /// Pops the last element from self
    pub fn pop(&mut self) -> T {
        use std::mem::MaybeUninit;
        let last_index = self.len;
        // This is safe, because res is swapped into the page and can only be overwritten, never read from.
        #[allow(clippy::uninit_assumed_init)]
        let mut res = unsafe { MaybeUninit::uninit().assume_init() };
        std::mem::swap(&mut self[last_index], &mut res);
        self.len -= 1;
        res
    }
    /// Clears the vector, removing all values.
    pub fn clear(&mut self) {
        self.drop_all();
        self.len = 0;
    }
    fn drop_all(&mut self) {
        use std::mem::MaybeUninit;
        for i in 0..self.len() {
            // This is safe, because tmp is swapped into the page, and then it is effectively forgotten.
            #[allow(clippy::uninit_assumed_init)]
            let mut tmp = unsafe { MaybeUninit::uninit().assume_init() };
            std::mem::swap(&mut self[i], &mut tmp);
        }
    }
}
impl<T: Sized> Drop for PagedVec<T> {
    fn drop(&mut self) {
        self.drop_all();
    }
}
impl<T: Sized> Deref for PagedVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data.get_ptr(0).cast::<T>(), self.len) }
    }
}
impl<T: Sized> DerefMut for PagedVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data.get_ptr_mut(0).cast::<T>(), self.len) }
    }
}
impl<T: Sized> Borrow<[T]> for PagedVec<T> {
    fn borrow(&self) -> &[T] {
        self
    }
}
impl<T: Sized> BorrowMut<[T]> for PagedVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_page_vec() {
        let mut vec: PagedVec<u64> = PagedVec::new(0x1000);
        assert!(vec.capacity() == 0x1000);
        for i in 0..vec.capacity() {
            vec.push_within_capacity(i as u64).expect("could not push!");
        }
    }
    #[test]
    fn test_page_vec_push() {
        let mut vec: PagedVec<u64> = PagedVec::new(0x1000);
        assert!(vec.capacity() == 0x1000);
        for i in 0..0x8000 {
            vec.push(i as u64);
        }
    }
    #[test]
    fn test_page_vec_drop() {
        let mut vec: PagedVec<String> = PagedVec::new(0x1000);
        assert!(vec.capacity() == 0x1000);
        for i in 0..vec.capacity() {
            vec.push_within_capacity("".to_owned())
                .expect("could not push!");
        }
    }
}
