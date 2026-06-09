#[cfg(test)]
mod tests {
    use super::super::pending_entries::*;
    use crate::constants::*;
    use anchor_lang::prelude::Pubkey;

    fn make_entry(buyer_seed: u8, amount: u64, slot: u64) -> LotteryEntry {
        let mut buyer_bytes = [0u8; 32];
        buyer_bytes[0] = buyer_seed;
        LotteryEntry {
            buyer: Pubkey::new_from_array(buyer_bytes),
            amount,
            slot,
        }
    }

    fn new_pending_entries() -> PendingEntries {
        PendingEntries {
            creator_mint: Pubkey::default(),
            head: 0,
            count: 0,
            overflow_count: 0,
            bump: 0,
            _padding: [0u8; 7],
            entries: [LotteryEntry::default(); MAX_PENDING_ENTRIES],
        }
    }

    fn read_entry(pe: &PendingEntries, logical_index: usize) -> &LotteryEntry {
        let max = MAX_PENDING_ENTRIES;
        let count = pe.count as usize;
        let head = pe.head as usize;
        let start = if count < max { 0 } else { head };
        let idx = (start + logical_index) % max;
        &pe.entries[idx]
    }

    fn relay(pe: &mut PendingEntries) -> Vec<LotteryEntry> {
        let count = pe.count as usize;
        if count == 0 {
            return Vec::new();
        }
        let max = MAX_PENDING_ENTRIES;
        let head = pe.head as usize;
        let start = if count < max { 0 } else { head };
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let idx = (start + i) % max;
            result.push(pe.entries[idx]);
        }
        pe.head = 0;
        pe.count = 0;
        result
    }

    #[test]
    fn test_push_single() {
        let mut pe = new_pending_entries();
        let entry = make_entry(1, 1000, 100);
        let was_full = pe.push(entry);

        assert!(!was_full);
        assert_eq!(pe.count, 1);
        assert_eq!(pe.head, 1);
        assert_eq!(pe.overflow_count, 0);
        assert_eq!(pe.entries[0].amount, 1000);
    }

    #[test]
    fn test_push_fills_buffer() {
        let mut pe = new_pending_entries();

        for i in 0..MAX_PENDING_ENTRIES {
            let was_full = pe.push(make_entry(i as u8, (i + 1) as u64 * 100, i as u64));
            assert!(!was_full, "Should not be full at index {}", i);
        }

        assert_eq!(pe.count as usize, MAX_PENDING_ENTRIES);
        assert_eq!(pe.head, 0); // wraps around
        assert_eq!(pe.overflow_count, 0);
    }

    #[test]
    fn test_push_overflow_drops_oldest() {
        let mut pe = new_pending_entries();

        for i in 0..MAX_PENDING_ENTRIES {
            pe.push(make_entry(i as u8, (i + 1) as u64 * 100, i as u64));
        }

        let was_full = pe.push(make_entry(255, 99999, 999));
        assert!(was_full, "Should be full (overflow)");
        assert_eq!(pe.count as usize, MAX_PENDING_ENTRIES);
        assert_eq!(pe.overflow_count, 1);
        assert_eq!(pe.head, 1);
        assert_eq!(pe.entries[0].amount, 99999);
    }

    #[test]
    fn test_push_multiple_overflows() {
        let mut pe = new_pending_entries();

        for i in 0..MAX_PENDING_ENTRIES {
            pe.push(make_entry(i as u8, 100, i as u64));
        }

        for i in 0..10u64 {
            let was_full = pe.push(make_entry(0, i + 1, 1000 + i));
            assert!(was_full);
        }

        assert_eq!(pe.overflow_count, 10);
        assert_eq!(pe.count as usize, MAX_PENDING_ENTRIES);
    }

    #[test]
    fn test_relay_empty() {
        let mut pe = new_pending_entries();
        let relayed = relay(&mut pe);
        assert!(relayed.is_empty());
    }

    #[test]
    fn test_relay_partial() {
        let mut pe = new_pending_entries();

        pe.push(make_entry(1, 100, 1));
        pe.push(make_entry(2, 200, 2));
        pe.push(make_entry(3, 300, 3));

        let relayed = relay(&mut pe);
        assert_eq!(relayed.len(), 3);
        assert_eq!(relayed[0].amount, 100);
        assert_eq!(relayed[1].amount, 200);
        assert_eq!(relayed[2].amount, 300);

        assert_eq!(pe.count, 0);
        assert_eq!(pe.head, 0);
    }

    #[test]
    fn test_relay_full_preserves_order() {
        let mut pe = new_pending_entries();

        for i in 0..MAX_PENDING_ENTRIES {
            pe.push(make_entry(i as u8, (i + 1) as u64 * 100, i as u64));
        }

        let relayed = relay(&mut pe);
        assert_eq!(relayed.len(), MAX_PENDING_ENTRIES);

        for (i, entry) in relayed.iter().enumerate() {
            assert_eq!(entry.amount, (i + 1) as u64 * 100);
        }
    }

    #[test]
    fn test_relay_after_overflow_gives_correct_order() {
        let mut pe = new_pending_entries();

        for i in 0..MAX_PENDING_ENTRIES {
            pe.push(make_entry(i as u8, (i + 1) as u64, i as u64));
        }

        pe.push(make_entry(0, 1001, 500));
        pe.push(make_entry(0, 1002, 501));
        pe.push(make_entry(0, 1003, 502));

        assert_eq!(pe.overflow_count, 3);

        let relayed = relay(&mut pe);
        assert_eq!(relayed.len(), MAX_PENDING_ENTRIES);

        assert_eq!(relayed[0].amount, 4);

        let last = relayed.len();
        assert_eq!(relayed[last - 3].amount, 1001);
        assert_eq!(relayed[last - 2].amount, 1002);
        assert_eq!(relayed[last - 1].amount, 1003);
    }

    #[test]
    fn test_relay_preserves_overflow_count() {
        let mut pe = new_pending_entries();

        for i in 0..MAX_PENDING_ENTRIES {
            pe.push(make_entry(i as u8, 100, i as u64));
        }
        pe.push(make_entry(0, 200, 999));
        assert_eq!(pe.overflow_count, 1);

        relay(&mut pe);

        assert_eq!(pe.overflow_count, 1);
        assert_eq!(pe.count, 0);
        assert_eq!(pe.head, 0);
    }

    #[test]
    fn test_needs_emergency_relay() {
        let mut pe = new_pending_entries();

        for i in 0..(EMERGENCY_RELAY_THRESHOLD - 1) {
            pe.push(make_entry(i as u8, 100, i as u64));
        }
        assert!(!pe.needs_emergency_relay());

        pe.push(make_entry(0, 100, 999));
        assert!(pe.needs_emergency_relay());
    }

    #[test]
    fn test_push_then_relay_then_push_again() {
        let mut pe = new_pending_entries();

        pe.push(make_entry(1, 100, 1));
        pe.push(make_entry(2, 200, 2));

        let relayed = relay(&mut pe);
        assert_eq!(relayed.len(), 2);

        pe.push(make_entry(3, 300, 3));
        assert_eq!(pe.count, 1);
        assert_eq!(pe.head, 1);
        assert_eq!(pe.entries[0].amount, 300);
    }
}
