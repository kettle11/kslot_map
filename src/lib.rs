#[cfg(feature = "linked_list_slot_map")]
mod linked_list_slot_map;
#[cfg(feature = "linked_list_slot_map")]
pub use linked_list_slot_map::*;

/// A data structure designed to efficiently store data with persistent IDs.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct SlotMap<T> {
    items: Vec<T>,
    item_to_indirection_index: Vec<usize>,
    indirection_indices: Vec<Entry>,
    free_indirection_indices: Vec<usize>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
struct Entry {
    item_index: usize,
    generation: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SlotMapHandle<T> {
    indirection_index: usize,
    generation: usize,
    phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> SlotMapHandle<T> {
    pub const fn from_index_and_generation(index: usize, generation: usize) -> Self {
        Self {
            indirection_index: index,
            generation,
            phantom: std::marker::PhantomData,
        }
    }

    pub const fn index_and_generation(&self) -> (usize, usize) {
        (self.indirection_index, self.generation)
    }
}

impl<T> std::hash::Hash for SlotMapHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.indirection_index.hash(state);
        self.generation.hash(state);
    }
}

impl<T> PartialEq for SlotMapHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.indirection_index == other.indirection_index
    }
}

impl<T> Eq for SlotMapHandle<T> {}

impl<T> PartialOrd for SlotMapHandle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for SlotMapHandle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.indirection_index.cmp(&other.indirection_index)
    }
}

impl<T> Copy for SlotMapHandle<T> {}

impl<T> Clone for SlotMapHandle<T> {
    fn clone(&self) -> Self {
        Self {
            indirection_index: self.indirection_index,
            generation: self.generation,
            phantom: self.phantom,
        }
    }
}

impl<T> core::fmt::Debug for SlotMapHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlotMapHandle")
            .field("indirection_index", &self.indirection_index)
            .field("generation", &self.generation)
            .finish()
    }
}

// This is safe because it does not actually contain a T.
unsafe impl<T> Send for SlotMapHandle<T> {}
unsafe impl<T> Sync for SlotMapHandle<T> {}

impl<T> SlotMap<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            indirection_indices: Vec::new(),
            item_to_indirection_index: Vec::new(),
            free_indirection_indices: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    pub fn iter_with_handle(&self) -> impl Iterator<Item = (&T, SlotMapHandle<T>)> {
        self.items.iter().enumerate().map(|(index, item)| {
            let indirection_index = self.item_to_indirection_index[index];
            let generation = self.indirection_indices[indirection_index].generation;
            (
                item,
                SlotMapHandle {
                    indirection_index,
                    generation,
                    phantom: std::marker::PhantomData,
                },
            )
        })
    }

    pub fn iter_mut_with_handle(&mut self) -> impl Iterator<Item = (&mut T, SlotMapHandle<T>)> {
        self.items.iter_mut().enumerate().map(|(index, item)| {
            let indirection_index = self.item_to_indirection_index[index];
            let generation = self.indirection_indices[indirection_index].generation;
            (
                item,
                SlotMapHandle {
                    indirection_index,
                    generation,
                    phantom: std::marker::PhantomData,
                },
            )
        })
    }

    pub fn next_handle(&self) -> SlotMapHandle<T> {
        let (indirection_index, generation) =
            if let Some(indirection_index) = self.free_indirection_indices.last() {
                let slot = &self.indirection_indices[*indirection_index];
                let generation = slot.generation + 1;

                (*indirection_index, generation)
            } else {
                let indirection_index = self.indirection_indices.len();
                (indirection_index, 0)
            };
        SlotMapHandle {
            indirection_index,
            generation,
            phantom: std::marker::PhantomData,
        }
    }

    fn new_handle_with_index(&mut self, item_index: usize) -> SlotMapHandle<T> {
        let (indirection_index, generation) =
            if let Some(indirection_index) = self.free_indirection_indices.pop() {
                let slot = &mut self.indirection_indices[indirection_index];
                let generation = slot.generation + 1;
                *slot = Entry {
                    item_index,
                    generation,
                };
                (indirection_index, generation)
            } else {
                let indirection_index = self.indirection_indices.len();
                self.indirection_indices.push(Entry {
                    item_index,
                    generation: 0,
                });
                (indirection_index, 0)
            };
        self.item_to_indirection_index.push(indirection_index);

        SlotMapHandle {
            indirection_index,
            generation,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn push(&mut self, item: T) -> SlotMapHandle<T> {
        let item_index = self.items.len();
        self.items.push(item);
        self.new_handle_with_index(item_index)
    }

    pub fn remove(&mut self, handle: SlotMapHandle<T>) -> Option<T> {
        let item_entry = self.indirection_indices.get_mut(handle.indirection_index)?;

        if handle.generation != item_entry.generation {
            return None;
        }

        // Increment to prevent future removes for the same handle from working.
        item_entry.generation += 1;

        let item_index = item_entry.item_index;
        self.indirection_indices[*self.item_to_indirection_index.last().unwrap()].item_index =
            item_index;
        let removed_item = self.items.swap_remove(item_index);
        self.item_to_indirection_index.swap_remove(item_index);
        self.free_indirection_indices.push(handle.indirection_index);
        Some(removed_item)
    }

    pub fn remove_unchecked_generation(&mut self, handle: SlotMapHandle<T>) -> Option<T> {
        let item_entry = self.indirection_indices.get_mut(handle.indirection_index)?;

        // Increment to prevent future removes for the same handle from working.
        item_entry.generation += 1;

        let item_index = item_entry.item_index;
        self.indirection_indices[*self.item_to_indirection_index.last().unwrap()].item_index =
            item_index;
        let removed_item = self.items.swap_remove(item_index);
        self.item_to_indirection_index.swap_remove(item_index);
        self.free_indirection_indices.push(handle.indirection_index);
        Some(removed_item)
    }

    pub fn get(&self, handle: SlotMapHandle<T>) -> Option<&T> {
        let entry = &self.indirection_indices[handle.indirection_index];
        if entry.generation != handle.generation {
            return None;
        }
        self.items.get(entry.item_index)
    }

    pub fn get_mut(&mut self, handle: SlotMapHandle<T>) -> Option<&mut T> {
        let entry = &self.indirection_indices[handle.indirection_index];
        if entry.generation != handle.generation {
            return None;
        }
        self.items.get_mut(entry.item_index)
    }

    /// Mutably access two separate handles.
    /// Returns (None, None) if handles overlap.
    pub fn get_mut_twice(
        &mut self,
        handle0: SlotMapHandle<T>,
        handle1: SlotMapHandle<T>,
    ) -> (Option<&mut T>, Option<&mut T>) {
        let entry0 = &self.indirection_indices[handle0.indirection_index];
        let entry1 = &self.indirection_indices[handle1.indirection_index];

        match entry0.item_index.cmp(&entry1.item_index) {
            std::cmp::Ordering::Less => {
                let (v0, v1) = self.items.split_at_mut(entry1.item_index);
                (v0.get_mut(entry0.item_index), v1.get_mut(0))
            }
            std::cmp::Ordering::Greater => {
                let (v0, v1) = self.items.split_at_mut(entry0.item_index);
                (v1.get_mut(0), v0.get_mut(entry1.item_index))
            }
            std::cmp::Ordering::Equal => return (None, None),
        }
    }

    pub fn get_unchecked_generation(&self, handle: SlotMapHandle<T>) -> Option<&T> {
        let entry = &self.indirection_indices[handle.indirection_index];
        self.items.get(entry.item_index)
    }

    pub fn get_mut_unchecked_generation(&mut self, handle: SlotMapHandle<T>) -> Option<&mut T> {
        let entry = &self.indirection_indices[handle.indirection_index];
        self.items.get_mut(entry.item_index)
    }
}
