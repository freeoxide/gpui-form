//! Dense touched-field bitset keyed by declaration-order field index.
//! Form-agnostic: the caller (generated `<Name>FormState` facade or
//! application code) supplies the path-to-index mapping and the size.

/// Dense bitset of touched/visited fields, keyed by declaration-order
/// index. Unknown indices are a no-op on every mutator and `false` on
/// every query.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TouchedFields {
    bits: Box<[u64]>,
    field_count: usize,
}

impl TouchedFields {
    /// Create a bitset sized for `field_count` fields, all untouched.
    ///
    /// Typically called with `<Name>FormPath::FIELD_COUNT`.
    pub fn new(field_count: usize) -> Self {
        let words = field_count.div_ceil(u64::BITS as usize);
        Self {
            bits: vec![0u64; words].into_boxed_slice(),
            field_count,
        }
    }

    /// The field count this bitset was sized for.
    pub fn field_count(&self) -> usize {
        self.field_count
    }

    /// Mark the field at `index` as touched. Unknown indices are a no-op.
    pub fn touch_index(&mut self, index: usize) {
        if let Some(bit) = self.bit_mask(index) {
            self.bits[index / u64::BITS as usize] |= bit;
        }
    }

    /// Whether the field at `index` is touched. Unknown indices are `false`.
    pub fn is_touched_index(&self, index: usize) -> bool {
        match self.bit_mask(index) {
            Some(bit) => self.bits[index / u64::BITS as usize] & bit != 0,
            None => false,
        }
    }

    /// Clear the touched flag for `index`. Unknown indices are a no-op.
    pub fn clear_index(&mut self, index: usize) {
        if let Some(bit) = self.bit_mask(index) {
            self.bits[index / u64::BITS as usize] &= !bit;
        }
    }

    /// Clear every touched flag.
    pub fn clear_all(&mut self) {
        for word in self.bits.iter_mut() {
            *word = 0;
        }
    }

    /// Iterate touched indices in ascending order.
    pub fn touched_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.field_count).filter(|&index| self.is_touched_index(index))
    }

    /// How many fields are touched.
    pub fn touched_count(&self) -> usize {
        self.touched_indices().count()
    }

    /// Whether any field is touched.
    pub fn is_any_touched(&self) -> bool {
        self.bits.iter().any(|word| *word != 0)
    }

    fn bit_mask(&self, index: usize) -> Option<u64> {
        if index >= self.field_count {
            return None;
        }
        Some(1u64 << (index % u64::BITS as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_untouched() {
        let touched = TouchedFields::new(4);
        assert_eq!(touched.field_count(), 4);
        assert!(!touched.is_any_touched());
        assert_eq!(touched.touched_count(), 0);
        assert_eq!(touched.touched_indices().count(), 0);
    }

    #[test]
    fn touch_and_query_single_index() {
        let mut touched = TouchedFields::new(4);
        touched.touch_index(2);
        assert!(touched.is_touched_index(2));
        assert!(!touched.is_touched_index(0));
        assert!(!touched.is_touched_index(1));
        assert!(!touched.is_touched_index(3));
        assert!(touched.is_any_touched());
        assert_eq!(touched.touched_count(), 1);
    }

    #[test]
    fn clear_index_removes_only_that_index() {
        let mut touched = TouchedFields::new(4);
        touched.touch_index(0);
        touched.touch_index(3);
        touched.clear_index(0);
        assert!(!touched.is_touched_index(0));
        assert!(touched.is_touched_index(3));
    }

    #[test]
    fn clear_all_removes_every_index() {
        let mut touched = TouchedFields::new(9);
        touched.touch_index(0);
        touched.touch_index(5);
        touched.touch_index(8);
        touched.clear_all();
        assert!(!touched.is_any_touched());
        assert_eq!(touched.touched_count(), 0);
    }

    #[test]
    fn touched_indices_iterate_in_ascending_order() {
        let mut touched = TouchedFields::new(70);
        touched.touch_index(69);
        touched.touch_index(0);
        touched.touch_index(33);
        assert_eq!(
            touched.touched_indices().collect::<Vec<_>>(),
            vec![0, 33, 69]
        );
        assert_eq!(touched.touched_count(), 3);
    }

    #[test]
    fn indices_span_word_boundaries() {
        let mut touched = TouchedFields::new(130);
        touched.touch_index(63);
        touched.touch_index(64);
        touched.touch_index(129);
        assert!(touched.is_touched_index(63));
        assert!(touched.is_touched_index(64));
        assert!(touched.is_touched_index(129));
        assert_eq!(touched.touched_count(), 3);
    }

    #[test]
    fn unknown_indices_are_no_ops() {
        let mut touched = TouchedFields::new(3);
        touched.touch_index(3);
        touched.touch_index(100);
        assert!(!touched.is_any_touched());
        assert!(!touched.is_touched_index(3));
        assert!(!touched.is_touched_index(100));
        touched.clear_index(9);
    }

    #[test]
    fn zero_fields_is_valid_and_inert() {
        let mut touched = TouchedFields::new(0);
        assert_eq!(touched.field_count(), 0);
        assert!(!touched.is_any_touched());
        touched.touch_index(0);
        assert!(!touched.is_touched_index(0));
        touched.clear_all();
        assert!(!touched.is_any_touched());
    }

    #[test]
    fn default_is_empty_and_untouched() {
        let touched = TouchedFields::default();
        assert_eq!(touched.field_count(), 0);
        assert!(!touched.is_any_touched());
    }

    #[test]
    fn clones_are_independent() {
        let mut touched = TouchedFields::new(2);
        touched.touch_index(1);
        let mut copy = touched.clone();
        copy.clear_index(1);
        assert!(touched.is_touched_index(1));
        assert!(!copy.is_touched_index(1));
        copy.touch_index(0);
        assert!(!touched.is_touched_index(0));
    }

    #[test]
    fn equality_is_by_bitset_content() {
        let mut a = TouchedFields::new(4);
        let mut b = TouchedFields::new(4);
        assert_eq!(a, b);
        a.touch_index(1);
        assert_ne!(a, b);
        b.touch_index(1);
        assert_eq!(a, b);
    }
}
