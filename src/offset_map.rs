use std::ops::Deref;
use std::slice::Iter;

pub struct OffsetMap {
    inner: Vec<(usize, OffsetType)>,
}

impl OffsetMap {
    pub fn new() -> OffsetMap {
        OffsetMap { inner: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> OffsetMap {
        OffsetMap { inner: Vec::with_capacity(capacity) }
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn insert(&mut self, offset: usize, offset_type: OffsetType) -> bool {
        if self.inner.len() > 0 {
            if offset > self.inner.last().unwrap().0 {
                self.inner.push((offset, offset_type));
                false
            } else if offset < self.inner.first().unwrap().0 {
                self.inner.insert(0, (offset, offset_type));
                false
            } else {
                match self.binary_search_by(|probe| probe.0.cmp(&offset)) {
                    Ok(_) => true,
                    Err(i) => {
                        self.inner.insert(i, (offset, offset_type));
                        false
                    }
                }
            }
        } else {
            self.inner.push((offset, offset_type));
            false
        }
    }

    pub fn get_offset(&self, offset: usize) -> Result<OffsetType, Option<(usize, OffsetType)>> {
        match self.binary_search_by(|probe| probe.0.cmp(&offset)) {
            Ok(i) => Ok(self.inner.get(i).unwrap().1),
            Err(i) => {
                debug_assert!(i <= self.inner.len());
                Err(if i == self.inner.len() {
                    None
                } else {
                    Some(*self.inner.get(i).unwrap())
                })
            }
        }
    }

    pub fn iter(&self) -> Iter<(usize, OffsetType)> {
        self.inner.iter()
    }
}

impl Deref for OffsetMap {
    type Target = Vec<(usize, OffsetType)>;

    fn deref(&self) -> &Vec<(usize, OffsetType)> {
        &self.inner
    }
}

#[derive(Copy, Debug, Clone)]
pub enum OffsetType {
    StringIdList,
    TypeIdList,
    PrototypeIdList,
    FieldIdList,
    MethodIdList,
    ClassDefList,
    ClassDef(usize),
    Map,
    TypeList,
    Type,
    AnnotationSetList,
    AnnotationSet,
    Annotation,
    AnnotationsDirectory,
    ClassData,
    Code,
    StringData,
    DebugInfo,
    EncodedArray,
    Link,
}
