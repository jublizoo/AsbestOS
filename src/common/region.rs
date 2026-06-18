use multiboot2::{MemoryArea, ElfSection};

/// A nonzero region of memory
#[derive(Copy, Clone, Debug)]
pub struct Region {
    start: *const u8,
    end: *const u8,
}

impl Region {
    pub fn try_new(start: *const u8, end: *const u8) -> Option<Self> {
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub fn new(start: *const u8, end: *const u8) -> Self {
        Self::try_new(start, end).unwrap()
    }

    pub fn start(&self) -> *const u8 {
        self.start
    }

    pub fn end(&self) -> *const u8 {
        self.end
    }

    pub fn size(&self) -> u64 {
        (self.end as u64) - (self.start as u64)
    }

    pub fn subtract(&self, other: &Region) -> [Option<Region>; 2] {
        let left = if self.start < other.start {
            Self::try_new(self.start, self.end.min(other.start))
        } else {
            None
        };
        let right = if other.end < self.end {
            Self::try_new(self.start.max(other.end), self.end)
        } else {
            None
        };


        [left, right]
    }

    pub fn intersects(&self, other: &Region) -> bool {
        other.start < self.end && other.end > self.start
    }
}

impl From<&MemoryArea> for Region {
    fn from(value: &MemoryArea) -> Self {
        Self::new(value.start_address() as *const u8, value.end_address() as *const u8)
    }
}


impl<'a> From<&ElfSection<'a>> for Region {
    fn from(value: &ElfSection) -> Self {
        Self::new(value.start_address() as *const u8, value.end_address() as *const u8)
    }
}
