use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem::{transmute, ManuallyDrop};
use std::ptr::NonNull;
use std::mem;
use std::ops::{Deref, Index, IndexMut};
use std::time::Instant;
#[allow(unused)]

#[derive(Debug)]
pub enum BorrowState{
    Exclusive,
    Shared(usize),
    Dropped,
}
pub type InnerVec<T> = ManuallyDrop<NonNull<UnsafeCell<Box<Vec<T>>>>>;

#[derive(Debug)]
pub struct Tangled<T>{
    //quick access for frequent elements
    cached: Vec<T>,

    //check the borrow state of a pointer
    borrow_state: UnsafeCell<HashMap<InnerVec<T>, (BorrowState, usize)>>, //usize is the index in the pointer_vec

    last_index: usize, //last index

    total_elements: usize,

    //list of stored pointers
    pointers: Vec<Option<InnerVec<T>>>,

    //keeps the size of each added vector, needed for log(n) random indexing via binary search
    prefix_vec: Vec<usize>,

}

pub struct BorrowedTangled<'t, T>{
    inner: &'t Tangled<T>
}

impl <'t, T> BorrowedTangled<'t, T>{
    fn get_ref(&self, index: (usize, usize)) -> Option<&'t T> {
        let small = index.0;
        let big = index.1;
        unsafe {
            let inner = self.inner.pointers[big - 1].unwrap();
            let vec = (&*inner.as_ptr()).get().as_ref().unwrap();
            return match vec.get(small) {
                Some(v) => Some(v),
                None => None
            }
        }
    }
}

pub struct MutBorrowedTangled<'t, T>{
    inner: &'t mut Tangled<T>
}
#[derive(Debug)]
pub struct RefHandle<'a, T> {
    ptr: InnerVec<T>,
    state: BorrowState,
    parent: NonNull<Tangled<T>>,
    _marker: PhantomData<&'a Tangled<T>>,
}

impl<'a, T: Debug> Deref for RefHandle<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let inner = unsafe { self.parent.as_ref() };
        let index = 0;
        let len = inner.cached.len();
        if index < len{
            return inner.cached.get(index).unwrap();
        }
        let index = index - len;
        println!("index: {:?}", index);
        let (offset, rough_index) = inner.convert_index(index).unwrap();
        println!("offset {}, rough_index {}", offset, rough_index);
        return self.get_element(offset).unwrap();
    }
}


impl<'a, T: std::fmt::Debug> RefHandle<'a, T>  {
    fn get_element(&self, index: usize) -> Option<&'a T> {
        unsafe {
            let parent = self.parent.as_ref();
            let (in_vec, _) = parent.convert_index(index)?;
            let ptr = &self.ptr;
            let vec = ptr.as_ptr().as_mut()?.get_mut();
            return vec.get(in_vec)
        }
    }
}

impl<T: std::fmt::Debug> Display for Tangled<T>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut display_vec = Vec::new();
        display_vec.push(Some(&self.cached));
        for i in self.pointers.iter(){
            if let Some(deref) = *i{
                let raw = unsafe {&*deref.as_ref()}.get();
                let x = unsafe {&*raw};
                display_vec.push(Some(x));
            }else {
                display_vec.push(None)
            };

        }
        write!(f, "{:?}", display_vec)
    }
}

pub struct TangledIter<'a, T>{
    pos: (usize, usize),
    last_len: usize,
    parent: &'a Tangled<T>,
    p_borrow: BorrowedTangled<'a, T>,
    unstable_iter: Vec<Option<InnerVec<T>>>,
    unstable_idx: Vec<usize>,
    _marker: PhantomData<&'a Tangled<T>>,
}

pub struct MutTangledIter<'a, T>{
    pos: (usize, usize),
    last_len: usize,
    p_borrow: MutBorrowedTangled<'a, T>,
    unstable_iter: Vec<Option<InnerVec<T>>>,
    unstable_idx: Vec<usize>,
    _marker: PhantomData<&'a Tangled<T>>,
}




impl<'a, T> TangledIter<'a, T> {
    fn new(parent: &'a Tangled<T>) -> TangledIter<'a, T> {
        return TangledIter{
            pos: (0,0),
            last_len: 0,
            unstable_idx: Vec::with_capacity(parent.total_elements),
            unstable_iter: Vec::with_capacity(parent.total_elements),
            parent,
            p_borrow: parent.borrow(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T> MutTangledIter<'a, T> {
    fn new(parent: &'a mut Tangled<T>) -> MutTangledIter<'a, T> {
        return MutTangledIter{
            pos: (0,0),
            last_len: 0,
            unstable_idx: Vec::with_capacity(parent.total_elements),
            unstable_iter: Vec::with_capacity(parent.total_elements),
            p_borrow: parent.borrow_mut(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, Idx: PartialEq<usize> + PartialOrd<usize> + Into<usize>> Index<(Idx, Idx)> for BorrowedTangled<'a, T> {
    type Output = T;
    fn index(&self, index: (Idx, Idx)) -> &Self::Output {
        let small = index.0;
        let big = index.1;
        if big == 0 && small <= self.inner.cached.len(){
            return &self.inner.cached[small.into()];
        }else {
            unsafe {
                let big_cpy = big == 1;
                let inner = self.inner.pointers[big.into() - if big_cpy {1} else {0} ].unwrap_or_else(|| panic!("index out of bounds"));
                let vec_ptr = inner.as_ptr();
                let vec = vec_ptr.as_ref().unwrap().get().as_ref().unwrap();
                return &vec[small.into()];

            }
        }
    }
}

impl<'a, T: Debug, Idx: PartialEq<usize> + PartialOrd<usize> + Into<usize>> Index<(Idx, Idx)> for MutBorrowedTangled<'a, T> {
    type Output = T;
    fn index(&self, index: (Idx, Idx)) -> &Self::Output {
        let small = index.0;
        let big = index.1;
        if big == 0 && small <= self.inner.cached.len(){
            return &self.inner.cached[small.into()];
        }else {
            unsafe {
                let big_cpy = big == 1;
                let inner = self.inner.pointers[big.into() - if big_cpy {1} else {0} ].unwrap_or_else(|| panic!("index out of bounds"));
                let vec_ptr = inner.as_ptr();
                let vec = vec_ptr.as_ref().unwrap().get().as_ref().unwrap();
                return &vec[small.into()];
            }
        }
    }
}

impl<'a, T: Debug, Idx: PartialEq<usize> + PartialOrd<usize> + Into<usize>> IndexMut<(Idx, Idx)> for MutBorrowedTangled<'a, T>{
    fn index_mut(&mut self, index: (Idx, Idx)) -> &mut T{
        let small = index.0;
        let big = index.1;
        if big == 0 && small <= self.inner.cached.len(){
            return &mut self.inner.cached[small.into()];
        }else {
            unsafe {
                let big_cpy = big == 1;
                let inner = self.inner.pointers[big.into() - if big_cpy {1} else {0} ].unwrap_or_else(|| panic!("index out of bounds"));
                let vec_ptr = inner.as_ptr();
                let vec = vec_ptr.as_ref().unwrap().get().as_mut().unwrap();
                return &mut vec[small.into()];

            }
        }
    }
}



impl<'p, T: Debug> Iterator for MutTangledIter<'p, T>{
    type Item = &'p mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let prefix = &self.p_borrow.inner.prefix_vec;
        let small = self.pos.0;

        if self.last_len < self.p_borrow.inner.cached.len() || self.p_borrow.inner.prefix_vec.len() == 1 {
            unsafe {
                if self.last_len >= self.p_borrow.inner.cached.len() {
                    return None;
                }
                let x = self.p_borrow.inner.cached.as_ptr() as *mut T;
                let ptr = x.byte_add(size_of::<T>() * self.last_len);
                self.last_len += 1;
                return ptr.as_mut();
            }
        }
        let big = self.pos.1 + 1;
        if big >= prefix.len(){
            return None;
        }
        let last = prefix[self.pos.1];
        let current = prefix[big];
        let difference = current - last;
        let diff_inverted: i64 = -(difference as i64);
        let small_idx = (diff_inverted + difference as i64 + small as i64) as usize;
        self.pos.0 += 1;
        if difference == small_idx + 1{
            self.pos.1 += 1;
            self.pos.0 = 0;
        }

        unsafe {
            if self.unstable_iter.is_empty(){
                let mut ptr = &mut self.p_borrow;
                return ptr.get_mut((small_idx, big))
            }
            let idx = self.unstable_idx[self.pos.0];
            let ptr = self.unstable_iter[big - 1].unwrap().as_ref();
            let inner = ptr.get().as_mut().unwrap();
            return Some(&mut inner[idx]);
        }


    }
}

impl<'p, T> Iterator for TangledIter<'p, T>{
    type Item = &'p T;
    fn next(&mut self) -> Option<Self::Item> {
        let prefix = &self.parent.prefix_vec;
        let small = self.pos.0;

        if self.last_len < self.parent.cached.len() || self.parent.prefix_vec.len() == 1 {
            let x = self.parent.cached.get(self.last_len);
            self.last_len += 1;
            return x;
        }
        let big = self.pos.1 + 1;
        if big >= prefix.len(){
            return None;
        }
        let last = prefix[self.pos.1];
        let current = prefix[big];
        let difference = current - last;
        let diff_inverted: i64 = -(difference as i64);
        let small_idx = (diff_inverted + difference as i64 + small as i64) as usize;
        self.pos.0 += 1;
        if difference == small_idx + 1{
            self.pos.1 += 1;
            self.pos.0 = 0;
        }
        unsafe {
            if self.unstable_iter.is_empty(){
                let ptr = &self.p_borrow;
                return ptr.get_ref((small_idx, big))
            }
            let idx = self.unstable_idx[self.pos.0];
            let ptr = self.unstable_iter[big - 1].unwrap().as_ref();
            let inner = ptr.get().as_ref().unwrap();
            return Some(&inner[idx]);

        }
    }
}

impl<T> Drop for RefHandle<'_, T> {
    fn drop(&mut self) {
        let old = mem::replace(&mut self.state, BorrowState::Shared(0));
        println!("drop called! {:?}", old);
        match old {
            BorrowState::Exclusive => {
                unsafe {
                    let p = self.parent.as_mut();
                    let (_, i) = unsafe {&*p.borrow_state.get()}.get(&self.ptr).unwrap();
                    {
                        unsafe { let _ = Box::from_raw(self.ptr.as_ptr()); }
                    }
                    p.pointers[*i] = None;
                }
            },

            BorrowState::Shared(0) => {
                unsafe {
                    let p = self.parent.as_mut();
                    let (_, i) = unsafe {&*p.borrow_state.get()}.get(&self.ptr).unwrap();
                    {
                        unsafe { let _ = Box::from_raw(self.ptr.as_ptr()); }
                    }
                    p.pointers[*i] = None;}
            },

            BorrowState::Shared(x) => {
                self.state = BorrowState::Shared(&x-1);

                let p = unsafe {self.parent.as_mut()};
                let (_, i) = unsafe {&*p.borrow_state.get()}.get(&self.ptr).unwrap();

                if x - 1 == 0{
                    p.pointers[*i] = None;
                    {
                        unsafe { let _ = Box::from_raw(self.ptr.as_ptr()); }
                    }

                }else {
                    //nothing really happens here
                }
            }

            BorrowState::Dropped => {
                {}
            }

        }
    }
}

impl<T> Tangled<T>{
    pub fn new() -> Tangled<T>{
        Tangled{
            cached: Vec::new(),
            borrow_state: UnsafeCell::new(HashMap::new()),
            last_index: 0,
            total_elements: 0,
            pointers: Vec::new(),
            prefix_vec: vec![0]
        }
    }
    pub fn from(v: Vec<T>) -> Tangled<T>{
        let len = v.len();
        Tangled{
            cached: v,
            borrow_state: UnsafeCell::new(HashMap::new()),
            last_index: 0,
            total_elements: len,
            pointers: Vec::new(),
            prefix_vec: vec![len]
        }
    }
    pub fn len(&self) -> usize{
        self.total_elements
    }

    pub fn iter(&self) -> TangledIter<'_, T> {
        TangledIter::new(self)
    }

    //may improve raw iteration performance at the cost of overhead via sorting
    //ideally initialize the iter prematurely it adds like 30%~ overhead
    pub fn unsorted_iter(&mut self) -> MutTangledIter<'_, T> {
        unsafe {
            let mut tangled = MutTangledIter::new(self);

            //clone kinda yuck but i don't wanna reorder the main struct
            let mut vec: Vec<usize> = tangled.p_borrow.inner.pointers.clone()
                .iter()
                .enumerate()
                .map(|(idx, prt)| {
                    tangled.unstable_idx.push(idx);
                    prt.unwrap().as_ref().get() as usize
                })
                .collect::<Vec<usize>>();
            vec.sort();

            assert_eq!(align_of::<Vec<Option<InnerVec<T>>>>(), align_of::<Vec<usize>>());

            //sound-ish? null pointer optimization should make sure it has the same layout... the align assert might also help
            let trs = transmute::<Vec<usize>, Vec<Option<InnerVec<T>>>>(vec);

            tangled.unstable_iter = trs;
            return tangled
        }
    }

    pub fn iter_mut(&mut self ) -> MutTangledIter<'_, T> {
        unsafe {MutTangledIter::new(self)}
    }

    pub fn push_fast(&mut self, val: T) {
        self.prefix_vec[0] += 1;
        self.cached.push(val);
    }

    pub fn pop_fast(&mut self) -> Option<T>{
        let popped = self.cached.pop();
        return if popped.is_some(){
            self.prefix_vec[0] -= 1;
            Some(popped.unwrap())
        }else {
            None
        }

    }


    pub fn borrow(&self) -> BorrowedTangled<'_, T> {
        return BorrowedTangled{inner: self};
    }

    pub fn borrow_mut(&mut self) -> MutBorrowedTangled<'_, T> {
        return MutBorrowedTangled{inner: self}
    }

    pub fn convert_index(&self, index: usize) -> Option<(usize, usize)>{
        if index < self.cached.len(){
            return Some((index, 0));
        }

        if index == self.total_elements{
            return None;
        }

        let target = index + 1;
        let rough_index = self.prefix_vec.partition_point(|&x| x < target);

        if rough_index >= self.total_elements{
            panic!("index out of bounds");
        }
        let offset = if rough_index == 0{
            index
        }else {
            index - self.prefix_vec[rough_index - 1]
        };
        return Some((offset, rough_index));
    }

    pub unsafe fn get_raw(&self, index: usize) -> Option<NonNull<T>> {

        let (offset, rough_index) = self.convert_index(index)?;

        let ptr = self.pointers[rough_index - 1];
        return match ptr.as_ref() {
            Some(ptr) => {
                let other = &(&*ptr.as_ref().get())[offset];
                Some(NonNull::from(other))
            },
            None => {
                None
            }
        }

    }

    pub fn vec_to_ptr(&self, vec: Vec<T>) -> InnerVec<T>{
        //stable heap address for the Cell
        let box_cell = Box::new(UnsafeCell::new(Box::new(vec)));

        //no memory leak because pointer gets saved and can be reconstructed and properly dropped
        let raw = Box::into_raw(box_cell);

        //SAFETY: NonNull::new_unchecked is fine because the vec cannot be null,
        //it was literally an owned value passed into the function

        let ptr = ManuallyDrop::new( unsafe { NonNull::new_unchecked(raw) });
        return ptr;
    }

}

impl<T: Debug> BorrowedTangled<'_, T>{
    fn get_handle(&self, index: usize) -> Option<RefHandle<'_, T>>{
        let ptr_at_index = match self.inner.pointers.get(index) {
            Some(Some(ptr)) => *ptr,
            None => return None,
            _ => unreachable!()
        };
        let raw = self.inner.borrow_state.get();

        let (index_copy, borrow_count) = {
            let entry = unsafe { &*raw }.get(&ptr_at_index).unwrap();
            let &(ref b, i) = entry;

            let borrow_count = match b {
                &BorrowState::Exclusive => panic!("Tangled already borrowed mutably"),
                &BorrowState::Shared(count) => count,
                &BorrowState::Dropped => panic!("indexing into dropped borrow!")
            };

            (i, borrow_count)
        };

        unsafe {&mut *raw}.insert(ptr_at_index, (BorrowState::Shared(&borrow_count + 1), index_copy));
        let ret = RefHandle{
            ptr: ptr_at_index,
            parent: NonNull::from(&*self.inner),
            state: BorrowState::Shared(borrow_count + 1),
            _marker: PhantomData
        };
        return Some(ret);
    }


    pub unsafe fn read_unchecked(&self, index: usize) -> Option<&T> {
        unsafe {
            return if let Some(ptr) = self.inner.get_raw(index).as_ref(){
                Some(ptr.as_ref())
            }else {
                None
            }
        }
    }

    pub fn read(&self, index: usize) -> Option<&T>  {
        let len = self.inner.cached.len();
        if index < len{
            return self.inner.cached.get(index);
        }
        let index = index - len;
        let (offset, rough_index) = self.inner.convert_index(index)?;
        println!("offset {}, rough_index {}", offset, rough_index);
        return if let Some(mut handle) = self.get_handle(rough_index){
            handle.get_element(offset)
        }else {
            None
        }

    }
}

impl<T: Debug> MutBorrowedTangled<'_, T> {

    pub fn push_vec(&mut self, vec: Vec<T>){
        let len = vec.len();

        let ptr = self.inner.vec_to_ptr(vec);
        {
            self.inner.pointers.push(Some(ptr));
        }

        let index = self.inner.last_index;

        unsafe {
            let raw = self.inner.borrow_state.get_mut();
            raw.insert(ptr, (BorrowState::Shared(0), index));
        }

        self.inner.last_index += 1;
        self.inner.total_elements += len;

        let last = self.inner.prefix_vec.last().unwrap();
        self.inner.prefix_vec.push(len + last);

    }
    fn get_mut_handle(&mut self, index: usize) -> Option<RefHandle<'_, T>>{
        let ptr_at_index = match self.inner.pointers.get(index) {
            Some(Some(ptr)) => *ptr,
            None => return None,
            _ => unreachable!()
        };
        let raw = self.inner.borrow_state.get_mut();


        let (index_copy, borrow_count) = {
            let entry = unsafe { &*raw }.get(&ptr_at_index).unwrap();
            let &(ref b, i) = entry;

            let borrow_count = match b {
                &BorrowState::Exclusive => panic!("Tangled already borrowed mutably"),
                &BorrowState::Shared(count) => count,
                &BorrowState::Dropped => panic!("indexing into dropped borrow!")
            };

            (i, borrow_count)
        };

        raw.insert(ptr_at_index, (BorrowState::Exclusive, index_copy));
        if borrow_count == 0{
            let ret = RefHandle{
                ptr: ptr_at_index,
                parent: NonNull::from(&*self.inner),
                state: BorrowState::Exclusive,
                _marker: PhantomData
            };
            return Some(ret)
        }else {
            panic!("cant borrow mutably");
        }
    }

    pub fn get_handle(&self, index: usize) -> Option<RefHandle<'_, T>>{
        let ptr_at_index = match self.inner.pointers.get(index) {
            Some(Some(ptr)) => *ptr,
            None => return None,
            x => {
                panic!("{:?}",x);
                unreachable!()
            }
        };
        let raw = self.inner.borrow_state.get();

        let (index_copy, borrow_count) = {
            let entry = unsafe { &*raw }.get(&ptr_at_index).unwrap();
            let &(ref b, i) = entry;

            let borrow_count = match b {
                &BorrowState::Exclusive => panic!("Tangled already borrowed mutably"),
                &BorrowState::Shared(count) => count,
                &BorrowState::Dropped => panic!("indexing into dropped borrow!")
            };

            (i, borrow_count)
        };


        unsafe {&mut *raw}.insert(ptr_at_index, (BorrowState::Shared(&borrow_count + 1), index_copy));
        let ret = RefHandle{
            ptr: ptr_at_index,
            parent: NonNull::from(&*self.inner),
            state: BorrowState::Shared(borrow_count + 1),
            _marker: PhantomData
        };
        return Some(ret);
    }
    pub unsafe fn drop_vec(&mut self, ptr: ManuallyDrop<NonNull<UnsafeCell<Box<Vec<T>>>>>) -> Option<usize>{ //returns the index it was dropped at
        let raw_borrow = self.inner.borrow_state.get_mut();

        match raw_borrow.get(&ptr) {
            Some((BorrowState::Exclusive, _)) => {
                let (_, i) = self.inner.borrow_state.get_mut().remove(&ptr).unwrap();
                self.inner.pointers[i] = None;
                {
                    let _ = Box::from_raw(ptr.as_ptr());
                }
                return Some(i)
            },
            Some((BorrowState::Shared(0), _)) => {
                let (_, i) = self.inner.borrow_state.get_mut().remove(&ptr).unwrap();
                self.inner.pointers[i] = None;
                {
                    let _ = Box::from_raw(ptr.as_ptr());
                }
                return Some(i)
            }
            Some((BorrowState::Shared(_), _)) => {
                return None
            },
            Some((BorrowState::Dropped, _)) => {
                return None
            }
            None => {
                return None
            }
        }
    }

    pub fn push(&mut self, val: T){
        if let Some(last) = self.inner.pointers.last_mut(){
            if let Some(inner) = last{
                unsafe {&mut *inner.as_mut().get()}.push(val);
                if let Some(non_empty) = self.inner.prefix_vec.last_mut(){
                    *non_empty += 1;
                }
            }
            else {
                panic!("vec is none!");
            }
        }else {
            panic!("pointers empty");
        }
    }

    pub unsafe fn read_unchecked(&self, index: usize) -> Option<&T> {
        unsafe {
            return if let Some(ptr) = self.inner.get_raw(index).as_ref(){
                Some(ptr.as_ref())
            }else {
                None
            }
        }
    }

    pub fn read(&'_ self, index: usize) -> RefHandle<'_, T> {
        let (offset,rough) = self.inner.convert_index(index).unwrap();
        //print!("offset {:?} rough {:?}", offset, rough);
        let handle = self.get_handle(rough).unwrap();
        let val = handle.get_element(index);
        //println!("val {:?}", val);
        //println!("handle {:?}", handle);
        todo!()
    }

    pub fn write(&mut self, index: usize, val: T){
        unsafe {
            if let Some(ptr) = self.inner.get_raw(index).as_mut(){
                ptr.replace(val);
            }
        }
    }

    pub fn alter<F>(&mut self, index: usize, pred: F) where F: Fn(&T) -> T{
        unsafe {
            if let Some(ptr) = self.inner.get_raw(index).as_mut(){
                let get_val = &*ptr.as_ptr();
                ptr.replace(pred(get_val));
            }
        }
    }


    pub fn get_pointer_at_index(&self, index: usize) -> Option<InnerVec<T>>{
        let rough = self.inner.prefix_vec.partition_point(|&x| x < index + 1);
        return self.inner.pointers[rough]
    }

    ///you should probably use the "get_pointer_at_index" method since it gives you the correct pointer first
    pub fn insert_vec(&mut self, pointer_index: usize, val: Vec<T>) -> Option<()>{
        let ptr = self.inner.vec_to_ptr(val);
        let inner = &mut self.inner.pointers;
        if inner[pointer_index].is_none(){
            inner[pointer_index] = Some(ptr);
            return Some(())
        }else {
            None
        }
    }
    fn get_mut<'p>(&mut self, index: (usize, usize)) -> Option<&'p mut T> {
        let small = index.0;
        let big = index.1;
        unsafe {
            let inner = self.inner.pointers[big - 1].unwrap();
            let vec = (&mut *inner.as_ptr()).get_mut().as_mut();
            return match vec.get_mut(small) {
                Some(v) => Some(v),
                None => None
            }
        }
    }

}
