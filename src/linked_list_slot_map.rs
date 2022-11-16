use crate::*;

/// A data structure that can accomodate multiple linked-lists stored within it.
pub struct LinkedListSlotMap<T> {
    slot_map: SlotMap<Node<T>>,
}

pub struct LinkedListSlotMapHandle<T>(pub(crate) SlotMapHandle<Node<T>>);

impl<T> LinkedListSlotMapHandle<T> {
    pub fn inner_handle(&self) -> SlotMapHandle<Node<T>> {
        self.0
    }
}
impl<T> Clone for LinkedListSlotMapHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for LinkedListSlotMapHandle<T> {}

impl<T> PartialEq for LinkedListSlotMapHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for LinkedListSlotMapHandle<T> {}

impl<T> LinkedListSlotMap<T> {
    pub fn new() -> Self {
        Self {
            slot_map: SlotMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        previous: Option<LinkedListSlotMapHandle<T>>,
        value: T,
    ) -> LinkedListSlotMapHandle<T> {
        let next = previous.and_then(|p| self.slot_map.get(p.0).unwrap().next);
        let new_handle = self.slot_map.push(Node {
            value,
            next,
            previous: previous.map(|p| p.0),
        });

        if let Some(previous) = previous {
            self.slot_map.get_mut(previous.0).unwrap().next = Some(new_handle);
        }
        if let Some(next) = next {
            self.slot_map.get_mut(next).unwrap().previous = Some(new_handle);
        }
        LinkedListSlotMapHandle(new_handle)
    }

    /// Returns the value and the previous and next nodes, if they exist.
    pub fn remove(
        &mut self,
        node: LinkedListSlotMapHandle<T>,
    ) -> (
        T,
        Option<LinkedListSlotMapHandle<T>>,
        Option<LinkedListSlotMapHandle<T>>,
    ) {
        let node = self.slot_map.remove(node.0).unwrap();
        if let Some(previous) = node.previous {
            self.slot_map.get_mut(previous).unwrap().next = node.next;
        }
        if let Some(next) = node.next {
            self.slot_map.get_mut(next).unwrap().previous = node.previous;
        }
        (
            node.value,
            node.previous.map(|p| LinkedListSlotMapHandle(p)),
            node.next.map(|p| LinkedListSlotMapHandle(p)),
        )
    }

    pub fn iter(&self, start_node: LinkedListSlotMapHandle<T>) -> LinkedListSlotMapIterator<T> {
        LinkedListSlotMapIterator {
            linked_list_slot_map: self,
            current_node: Some(start_node.0),
        }
    }

    pub fn get(&self, handle: LinkedListSlotMapHandle<T>) -> Option<&T> {
        self.slot_map.get(handle.0).map(|n| &n.value)
    }

    pub fn get_mut(&mut self, handle: LinkedListSlotMapHandle<T>) -> Option<&mut T> {
        self.slot_map.get_mut(handle.0).map(|n| &mut n.value)
    }

    pub fn get_previous_handle(
        &self,
        handle: LinkedListSlotMapHandle<T>,
    ) -> Option<LinkedListSlotMapHandle<T>> {
        Some(LinkedListSlotMapHandle(
            self.slot_map.get(handle.0)?.previous?,
        ))
    }

    pub fn get_next_handle(
        &self,
        handle: LinkedListSlotMapHandle<T>,
    ) -> Option<LinkedListSlotMapHandle<T>> {
        Some(LinkedListSlotMapHandle(self.slot_map.get(handle.0)?.next?))
    }

    pub fn reverse_iter(
        &self,
        start_node: LinkedListSlotMapHandle<T>,
    ) -> RevLinkedListSlotMapIterator<T> {
        RevLinkedListSlotMapIterator {
            linked_list_slot_map: self,
            current_node: Some(start_node.0),
        }
    }

    /// Reverse-iterate the linked-list and remove the values at the same time.
    pub fn reverse_remove_iter(
        &mut self,
        start_node: LinkedListSlotMapHandle<T>,
    ) -> RevRemoveLinkedListSlotMapIterator<T> {
        RevRemoveLinkedListSlotMapIterator {
            linked_list_slot_map: self,
            current_node: Some(start_node.0),
        }
    }
}

pub struct LinkedListSlotMapIterator<'a, T> {
    linked_list_slot_map: &'a LinkedListSlotMap<T>,
    current_node: Option<SlotMapHandle<Node<T>>>,
}
impl<'a, T> Iterator for LinkedListSlotMapIterator<'a, T> {
    type Item = (&'a T, LinkedListSlotMapHandle<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let node_handle = self.current_node?;
        let node = self.linked_list_slot_map.slot_map.get(node_handle).unwrap();
        self.current_node = node.next;
        Some((&node.value, LinkedListSlotMapHandle(node_handle)))
    }
}

pub struct RevLinkedListSlotMapIterator<'a, T> {
    linked_list_slot_map: &'a LinkedListSlotMap<T>,
    current_node: Option<SlotMapHandle<Node<T>>>,
}
impl<'a, T> Iterator for RevLinkedListSlotMapIterator<'a, T> {
    type Item = (&'a T, LinkedListSlotMapHandle<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let node_handle = self.current_node?;

        let node = self.linked_list_slot_map.slot_map.get(node_handle).unwrap();
        self.current_node = node.previous;
        Some((&node.value, LinkedListSlotMapHandle(node_handle)))
    }
}

pub struct RevRemoveLinkedListSlotMapIterator<'a, T> {
    linked_list_slot_map: &'a mut LinkedListSlotMap<T>,
    current_node: Option<SlotMapHandle<Node<T>>>,
}
impl<'a, T> Iterator for RevRemoveLinkedListSlotMapIterator<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let node_handle = self.current_node?;

        let node = self
            .linked_list_slot_map
            .slot_map
            .remove(node_handle)
            .unwrap();
        self.current_node = node.previous;
        Some(node.value)
    }
}

pub struct Node<T> {
    value: T,
    next: Option<SlotMapHandle<Node<T>>>,
    previous: Option<SlotMapHandle<Node<T>>>,
}
