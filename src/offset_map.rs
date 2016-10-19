use std::ops::{Deref, DerefMut};

pub struct OffsetMap {
    inner: Vec<(u32, OffsetType)>,
}

impl OffsetMap {
    /// Creates a new offset map with the given initial capacity. It will allocate space for
    /// exactly `capacity` offsets.
    pub fn with_capacity(capacity: usize) -> OffsetMap {
        OffsetMap { inner: Vec::with_capacity(capacity) }
    }

    /// Inserts a new offset in the offset map.
    pub fn insert(&mut self, offset: u32, offset_type: OffsetType) -> bool {
        if !self.inner.is_empty() {
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

    /// Gets the given offset, if it exists at the map. The parameter is the offset from the start
    /// of the file, and it will be searched in the stored offsets in the map.
    pub fn get_offset(&self, offset: u32) -> Result<OffsetType, Option<(u32, OffsetType)>> {
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
}

impl Deref for OffsetMap {
    type Target = Vec<(u32, OffsetType)>;

    fn deref(&self) -> &Vec<(u32, OffsetType)> {
        &self.inner
    }
}

impl DerefMut for OffsetMap {
    fn deref_mut(&mut self) -> &mut Vec<(u32, OffsetType)> {
        &mut self.inner
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum OffsetType {
    StringIdList,
    TypeIdList,
    PrototypeIdList,
    FieldIdList,
    MethodIdList,
    ClassDefList,
    Map,
    TypeList,
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
