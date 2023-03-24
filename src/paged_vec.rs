// All functions properly documented, with examples!
use crate::Pages;
use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
/// A [`Vec`]-like type located in memory pages acquired directly from the kernel. For big lengths a faster to
/// allocate/deallocate than a normal [`Vec`], but considerably slower for small sizes. Intended to be used for very large data
/// sets, with a rough estimate of capacity known ahead of time.
/// # Advantages:
/// 1. 2-3x times faster than default allocator for big vec sizes (over ~20 MB).
/// 2. memory is released directly to the kernel as soon as [`PagedVec`] is dropped, which may not always be the case for
/// standard allocator, leading to decreased memory footprint.
// 3. More conservative growth model. Since [`PagedVec`] is intended for very large sizes, it is considerably more conservative with
// allocating memory(1.5x previous cap instead of 2x for standard [`Vec`].
/// # Disadvantages
/// 1. Slower to realocate for small data sets
/// 2. Can't be turned into a `Box<[T]>`
/// # Examples
/// Some examples/documentation for functions of this type are derived from examples for [`Vec`] in rust standard library, to
/// better highlight the differences and similarities.
pub struct PagedVec<T: Sized> {
    data: Pages<crate::AllowRead, crate::AllowWrite, crate::DenyExec>,
    len: usize,
    pd: PhantomData<T>,
}
impl<T: Sized> PagedVec<T> {
    /// Creates a new [`PagedVec`] with specified `capacity`.
    /// # Examples
    /// ```
    /// # use memory_pages::*;
    /// // capacity must be specified!
    /// let mut vec = PagedVec::new(0x1000);
    /// vec.push_within_capacity(0.0).unwrap();
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
    /// An alias for [`Self::new`] provided for compatibility purposes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(capacity)
    }
    /// Pushes `t` into `self` if under capacity, else returns `t`.
    /// # Examples
    /// ```
    /// # use memory_pages::*;
    /// let mut vec = PagedVec::new(0x1000);
    /// // Push is within capacity, OK!
    /// vec.push_within_capacity(0.0).unwrap();
    /// for _ in 0..(vec.capacity() - 1){
    ///     vec.push_within_capacity(1.23).unwrap();
    /// }
    /// // push outside capacity, pushed value returned!
    /// assert_eq!(vec.push_within_capacity(5.6),Err(5.6));
    #[must_use]
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
    /// Advises this [`PagedVec`] that `used` elements are going to be in use soon.
    /// # Beware
    /// Usage hints are part of fine-grain memory access adjustments. It is *NOT* always beneficial to use, in
    /// contrary, it very often slows allocations down. Before using them, test each usage.
    pub fn advise_use_soon(&mut self, used: usize) {
        if self.len() < used {
            self.resize(used);
        }
        self.data.advise_use_soon(used);
    }
    /// Advises this [`PagedVec`] that it is going to be accessed sequentially.
    /// # Beware
    /// Usage hints are part of fine-grain memory access adjustments. It is *NOT* always beneficial to use, in
    /// contrary, it very often slows allocations down. Before using them, test each usage.
    pub fn advise_use_seq(&mut self) {
        self.data.advise_use_seq();
    }
    /// Advises this [`PagedVec`] that it is going to be accessed randomly.
    /// # Beware
    /// Usage hints are part of fine-grain memory access adjustments. It is *NOT* always beneficial to use, in
    /// contrary, it very often slows allocations down. Before using them, test each usage.
    pub fn advise_use_rnd(&mut self) {
        self.data.advise_use_rnd();
    }
    fn get_next_cap(cap: usize) -> usize {
        //(cap + cap / 2).max(0x1000)
        cap * 2
    }
    fn resize(&mut self, next_cap: usize) {
        let bytes_cap = next_cap * std::mem::size_of::<T>();
        self.data.resize(bytes_cap);
        /*
        let cpy_len = self.len() * std::mem::size_of::<T>();
        let mut data = Pages::new(bytes_cap);
        data.split_at_mut(cpy_len)
            .0
            .copy_from_slice(self.data.split_at_mut(cpy_len).0);
        self.data = data;
        */
    }
    /// Reserves capacity for at least additional more elements to be inserted in the given [`PagedVec<T>`]. The collection may
    /// reserve more space to speculatively avoid frequent reallocations. After calling reserve, capacity will be greater than
    /// or equal to self.len() + additional. Does nothing if capacity is already sufficient.
    /// # Examples
    /// ```
    /// # use memory_pages::*;
    /// let mut vec:PagedVec<u8> = PagedVec::new(0x4000);
    /// let init_cap = vec.capacity();
    /// // Capacity was less or equal to current capacity, no need to reallocate.
    /// vec.reserve(0x4000);
    /// assert_eq!(init_cap,vec.capacity());
    /// // Requested higher capacity, a reallocation may occur!
    /// vec.reserve(0x8000);
    /// assert!(init_cap<vec.capacity());
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        if self.len() + additional <= self.capacity() {
            return;
        };
        self.resize((self.len() + additional).max(Self::get_next_cap(self.capacity())));
    }
    /// Reserves the minimum capacity for at least additional more elements to be inserted in the given [`PagedVec<T>`]. Unlike
    /// reserve, this will not deliberately over-allocate to speculatively avoid frequent allocations. After calling
    /// [`Self::reserve_exact`], capacity will be greater than or equal to self.len() + additional. Does nothing if the capacity is
    /// already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied upon
    /// to be precisely minimal. Using reserve before [`Self::push`] is preferred over using just [`Self::push`], because
    /// reallocation's of [`PagedVec`] are slow.
    /// # Examples
    /// ```
    /// # use memory_pages::*;
    /// let mut vec:PagedVec<u8> = PagedVec::new(0x4000);
    /// let init_cap = vec.capacity();
    /// // Capacity was less or equal to current capacity, no need to reallocate.
    /// vec.reserve_exact(0x4000);
    /// assert_eq!(init_cap,vec.capacity());
    /// // Requested higher capacity, a reallocation may occur!
    /// vec.reserve_exact(0x8000);
    /// assert!(init_cap<vec.capacity());
    /// ```
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
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// # use memory_pages::PagedVec;
    /// let mut v = PagedVec::new(3);
    /// v.push(1);
    /// v.push(2);
    /// v.push(3);
    /// # v[2] = 3 as u8;
    /// assert_eq!(v.remove(1), 2);
    /// let slice:&[u8] = &[1, 3];
    /// assert_eq!(v,slice);
    /// ```
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
    /// Pushes `t` into `self` if under capacity, else returns `t`.
    /// # Examples
    /// ```
    /// # use memory_pages::*;
    /// let mut vec = PagedVec::new(0x1000);
    /// // Push is within capacity, no realocations!
    /// vec.push(0.0);
    /// for _ in 0..(vec.capacity() - 1){
    ///     vec.push(1.23);
    /// }
    /// // push outside capacity, a slow reallocation occurs, but `push` still succeeds!
    /// vec.push(5.6);
    pub fn push(&mut self, t: T) {
        if self.len * std::mem::size_of::<T>() >= self.data.len() {
            self.resize(Self::get_next_cap(self.capacity()));
        }
        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            std::ptr::write(end, t);
            self.len += 1;
        };
    }
    /// Gets the capacity of `self`.
    /// ```
    /// # use memory_pages::*;
    /// let mut vec = PagedVec::new(0x1000);
    /// // `vec` can store `cap` items in total.
    /// let cap = vec.capacity();
    /// let remaining = vec.capacity() - vec.len();
    /// // `push_within_capacity` `remaining` times will succeed.
    /// for _ in 0..remaining{
    ///     vec.push_within_capacity(0.5).unwrap();
    /// }
    /// // No more space left!
    /// assert_eq!(vec.capacity() - vec.len(),0);
    /// // pushing over capacity will fail.
    /// assert_eq!(vec.push_within_capacity(0.7),Err(0.7));
    /// ```
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.len() / std::mem::size_of::<T>()
    }
    /// Pops the last element from `self`
    /// ```
    /// # use memory_pages::*;
    /// let mut vec = PagedVec::new(0x1000);
    /// vec.push(0);
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// assert_eq!(vec.pop(),Some(3));
    /// assert_eq!(vec.pop(),Some(2));
    /// assert_eq!(vec.pop(),Some(1));
    /// assert_eq!(vec.pop(),Some(0));
    /// assert_eq!(vec.pop(),None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        use std::mem::MaybeUninit;
        if self.len == 0 {
            return None;
        }
        let last_index = self.len - 1;
        // This is safe, because res is swapped into the page and can only be overwritten, never read from.
        #[allow(clippy::uninit_assumed_init)]
        let mut res = unsafe { MaybeUninit::uninit().assume_init() };
        std::mem::swap(&mut self[last_index], &mut res);
        self.len -= 1;
        Some(res)
    }
    /// Clears the vector, removing all values.
    /// # Examples
    /// ```
    /// # use memory_pages::*;
    /// let mut vec = PagedVec::new(0x1000);
    /// vec.push(8);
    /// let cap = vec.capacity();
    /// vec.clear();
    /// // After clearing length is 0
    /// assert_eq!(vec.len(),0);
    /// // But the capacity does not change!
    /// assert_eq!(vec.capacity(),cap);
    /// ```
    pub fn clear(&mut self) {
        self.drop_all();
        self.len = 0;
    }
    /// Works exacly the same as [`Self::clear`] but hints the OS that some of the memory occupied by data inside this 
    /// [`PagedVec`] is going to be unused, allowing it to be temporarily reclaimed. This allows the memory to be 
    /// reserved, but not backed by physical RAM until next use, reducing RAM usage.
    pub fn clear_decommit(&mut self){
        self.clear();
        self.data.decommit(0, self.data.len());
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
use std::fmt::{Debug, Formatter};
impl<T: Debug> Debug for PagedVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&**self, f)
    }
}
impl<T: PartialEq> PartialEq<[T]> for PagedVec<T> {
    fn eq(&self, other: &[T]) -> bool {
        self[..] == other[..]
    }
}
impl<T: PartialEq> PartialEq<&[T]> for PagedVec<T> {
    fn eq(&self, other: &&[T]) -> bool {
        self[..] == (*other)[..]
    }
}
impl<T: PartialEq> PartialEq<Vec<T>> for PagedVec<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self[..] == other[..]
    }
}
impl<T: Clone> Clone for PagedVec<T> {
    fn clone(&self) -> Self {
        let mut cloned = Self::new(self.capacity());
        for t in self {
            cloned.push(t.clone());
        }
        cloned
    }
}
impl<'a, T> IntoIterator for &'a PagedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
