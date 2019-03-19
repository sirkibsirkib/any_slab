
use std::{any::TypeId, mem};
use hashbrown::HashMap;

#[cfg(test)]
mod tests;

struct Hole {
    from: usize,
    to: usize,
}
impl Hole {
    fn len(&self) -> usize {
        self.to - self.from
    }
    fn everything() -> Self {
        Self {
            from: 0,
            to: std::usize::MAX,
        }
    }
}

pub struct AnySlab {
    holes: Vec<Hole>,
    allocated: HashMap<usize, TypeId>,
    data: Vec<u8>,
}

#[derive(Copy, Clone, Debug)]
pub enum AnySlabError {
    TypeMismatch,
    UnknownKey,
}

impl Default for AnySlab {
    fn default() -> Self {
        Self {
            holes: vec![Hole::everything()],
            allocated: HashMap::default(),
            data: vec![],
        }
    }
}
impl AnySlab {
    pub fn clear(&mut self) {
        self.allocated.clear();
        self.holes.clear();
        self.holes.push(Hole::everything());
    }
    pub fn insert<T: 'static + Sized>(&mut self, t: T) -> usize {
        let bytes_needed = mem::size_of::<T>();

        // 1. find the first hole that has sufficient size (1+ must exist)
        let (hold_idx, hole) = self
            .holes
            .iter_mut()
            .enumerate()
            .find(|(_idx, hole)| hole.len() >= bytes_needed)
            .unwrap();

        // 2. extend vector while data hangs over edge
        let key = hole.from;
        let key_end = key + bytes_needed;
        while self.data.len() < key_end {
            self.data.push(0);
        }

        // 3. modify hole. move to end (so its not repeatedly checked) possibly discard
        hole.from = key_end;
        let this_hole = self.holes.swap_remove(hold_idx);
        if this_hole.len() > 0 {
            self.holes.push(this_hole);
        }

        // 4. write data
        let slice = &mut self.data[key];
        unsafe {
            let existing: &mut T = mem::transmute(slice);
            let got_prev = mem::replace(existing, t);
            mem::forget(got_prev);
        };
        self.allocated.insert(key, TypeId::of::<T>());
        key
    }
    fn verify_key<T: 'static + Sized>(&self, key: usize) -> Result<(), AnySlabError> {
        let requested_tid = TypeId::of::<T>();
        match self.allocated.get(&key) {
            Some(&key_tid) if key_tid == requested_tid => Ok(()),
            Some(_) => Err(AnySlabError::TypeMismatch),
            None => Err(AnySlabError::UnknownKey),
        }
    }
    pub fn iter<T: 'static + Sized>(&self) -> impl Iterator<Item = (usize, &T)> {
        let requested_tid = TypeId::of::<T>();
        self.allocated
            .iter()
            .filter(move |(_key, &tid)| tid == requested_tid)
            .map(move |(&key, _tid)| (key, unsafe { mem::transmute(&self.data[key]) }))
    }
    pub fn iter_mut<T: 'static + Sized>(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        let requested_tid = TypeId::of::<T>();
        let AnySlab {
            allocated, data, ..
        } = self;
        allocated
            .iter()
            .filter(move |(_key, &tid)| tid == requested_tid)
            .map(move |(&key, _tid)| (key, unsafe { mem::transmute(&mut data[key]) }))
    }
    pub fn len(&self) -> usize {
        self.allocated.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn contains_key<T: 'static + Sized>(&self, key: usize) -> bool {
        self.verify_key::<T>(key).is_ok()
    }
    pub fn remove<T: 'static + Sized>(&mut self, key: usize) -> Result<T, AnySlabError> {
        // 1. check if valid key & type
        self.verify_key::<T>(key)?;
        let bytes_needed = mem::size_of::<T>();
        let key_end = key + bytes_needed;

        // 2. update holes
        let mut new_hole = Hole {
            from: key,
            to: key + mem::size_of::<T>(),
        };
        {
            // 2cont. fuse holes with left & right
            let mut lid = None;
            let mut rid = None;
            for (i, hole) in self.holes.iter().enumerate() {
                if hole.to == key {
                    lid = Some(i);
                    if rid.is_some() {break}
                } else if hole.from == key_end {
                    rid = Some(i);
                    if lid.is_some() {break}
                }
            }
            if let Some(lid) = lid {
                let l_hole: &mut Hole = unsafe { self.holes.get_unchecked_mut(lid) };
                new_hole.from = l_hole.from;
                self.holes.swap_remove(lid);
                if let Some(ref mut rid) = rid {
                    if *rid == self.holes.len() {
                        // rid got swapped into LID's position!
                        *rid = lid;
                    }
                }
            }
            if let Some(rid) = rid {
                let r_hole: &mut Hole = unsafe { self.holes.get_unchecked_mut(rid) };
                new_hole.to = r_hole.to;
                self.holes.swap_remove(rid);
            }
        }
        self.holes.push(new_hole);

        // 3. remove allocation
        self.allocated.remove(&key);

        // 4. return the datum
        let slice = &mut self.data[key];
        unsafe {
            let existing: &mut T = mem::transmute(slice);
            let mut t: T = mem::uninitialized();
            let got_prev = mem::swap(&mut t, existing);
            mem::forget(got_prev);
            Ok(t)
        }
    }
    pub fn get<T: 'static + Sized>(&self, key: usize) -> Result<&T, AnySlabError> {
        // 1. check if valid key & type
        self.verify_key::<T>(key)?;

        // 2. return the reference
        Ok(unsafe { mem::transmute(&self.data[key]) })
    }

    pub fn get_mut<T: 'static + Sized>(&mut self, key: usize) -> Result<&mut T, AnySlabError> {
        // 1. check if valid key & type
        self.verify_key::<T>(key)?;

        // 2. return the reference
        Ok(unsafe { mem::transmute(&mut self.data[key]) })
    }
}
