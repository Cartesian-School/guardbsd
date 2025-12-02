// tests/integration/capability_integration.rs
// Integration Tests for Capability System
// ============================================================================

#[cfg(test)]
mod capability_integration_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct CapId(u64);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Rights(u32);

    impl Rights {
        const READ: u32 = 1 << 0;
        const WRITE: u32 = 1 << 1;
        const GRANT: u32 = 1 << 3;

        fn new(bits: u32) -> Self { Self(bits) }
        fn has(&self, other: u32) -> bool { (self.0 & other) == other }
        fn subset(&self, other: Rights) -> bool { (self.0 & other.0) == self.0 }
    }

    struct MockCapabilitySystem {
        caps: Arc<Mutex<HashMap<CapId, (u64, Rights)>>>,
        next_id: Arc<Mutex<u64>>,
    }

    impl MockCapabilitySystem {
        fn new() -> Self {
            Self {
                caps: Arc::new(Mutex::new(HashMap::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }

        fn create_cap(&self, object_id: u64, rights: Rights) -> CapId {
            let mut next = self.next_id.lock().unwrap();
            let id = CapId(*next);
            *next += 1;
            
            let mut caps = self.caps.lock().unwrap();
            caps.insert(id, (object_id, rights));
            id
        }

        fn get_cap(&self, id: CapId) -> Option<(u64, Rights)> {
            self.caps.lock().unwrap().get(&id).copied()
        }

        fn delegate(&self, id: CapId, new_rights: Rights) -> Option<CapId> {
            let (obj_id, rights) = {
                let caps = self.caps.lock().unwrap();
                let (obj_id, rights) = caps.get(&id)?;
                
                if !rights.has(Rights::GRANT) || !new_rights.subset(*rights) {
                    return None;
                }
                (*obj_id, *rights)
            };
            
            Some(self.create_cap(obj_id, new_rights))
        }

        fn revoke(&self, id: CapId) -> bool {
            self.caps.lock().unwrap().remove(&id).is_some()
        }

        fn cap_count(&self) -> usize {
            self.caps.lock().unwrap().len()
        }
    }

    #[test]
    fn test_capability_creation() {
        let system = MockCapabilitySystem::new();
        let cap = system.create_cap(42, Rights::new(Rights::READ));
        
        let (obj_id, rights) = system.get_cap(cap).unwrap();
        assert_eq!(obj_id, 42);
        assert!(rights.has(Rights::READ));
    }

    #[test]
    fn test_capability_delegation() {
        let system = MockCapabilitySystem::new();
        let parent = system.create_cap(100, Rights::new(Rights::READ | Rights::GRANT));
        
        let child = system.delegate(parent, Rights::new(Rights::READ));
        assert!(child.is_some());
        
        let (obj_id, rights) = system.get_cap(child.unwrap()).unwrap();
        assert_eq!(obj_id, 100);
        assert!(rights.has(Rights::READ));
        assert!(!rights.has(Rights::GRANT));
    }

    #[test]
    fn test_delegation_without_grant() {
        let system = MockCapabilitySystem::new();
        let cap = system.create_cap(1, Rights::new(Rights::READ));
        
        let result = system.delegate(cap, Rights::new(Rights::READ));
        assert!(result.is_none());
    }

    #[test]
    fn test_delegation_rights_escalation() {
        let system = MockCapabilitySystem::new();
        let cap = system.create_cap(1, Rights::new(Rights::READ | Rights::GRANT));
        
        let result = system.delegate(cap, Rights::new(Rights::READ | Rights::WRITE));
        assert!(result.is_none());
    }

    #[test]
    fn test_capability_revocation() {
        let system = MockCapabilitySystem::new();
        let cap = system.create_cap(1, Rights::new(Rights::READ));
        
        assert!(system.revoke(cap));
        assert!(system.get_cap(cap).is_none());
    }

    #[test]
    fn test_multiple_delegations() {
        let system = MockCapabilitySystem::new();
        let root = system.create_cap(1, Rights::new(Rights::READ | Rights::WRITE | Rights::GRANT));
        
        let child1 = system.delegate(root, Rights::new(Rights::READ | Rights::GRANT)).unwrap();
        let child2 = system.delegate(child1, Rights::new(Rights::READ)).unwrap();
        
        let (_, rights) = system.get_cap(child2).unwrap();
        assert!(rights.has(Rights::READ));
        assert!(!rights.has(Rights::WRITE));
        assert!(!rights.has(Rights::GRANT));
    }

    #[test]
    fn test_capability_isolation() {
        let system = MockCapabilitySystem::new();
        let cap1 = system.create_cap(100, Rights::new(Rights::READ));
        let cap2 = system.create_cap(200, Rights::new(Rights::WRITE));
        
        let (obj1, _) = system.get_cap(cap1).unwrap();
        let (obj2, _) = system.get_cap(cap2).unwrap();
        
        assert_ne!(obj1, obj2);
    }

    #[test]
    fn test_concurrent_capability_creation() {
        use std::thread;
        
        let system = Arc::new(MockCapabilitySystem::new());
        let mut handles = vec![];
        
        for i in 0..10 {
            let sys = Arc::clone(&system);
            let handle = thread::spawn(move || {
                sys.create_cap(i, Rights::new(Rights::READ));
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(system.cap_count(), 10);
    }

    #[test]
    fn test_delegation_chain() {
        let system = MockCapabilitySystem::new();
        let root = system.create_cap(1, Rights::new(Rights::READ | Rights::WRITE | Rights::GRANT));
        
        let level1 = system.delegate(root, Rights::new(Rights::READ | Rights::GRANT)).unwrap();
        let level2 = system.delegate(level1, Rights::new(Rights::READ | Rights::GRANT)).unwrap();
        let level3 = system.delegate(level2, Rights::new(Rights::READ)).unwrap();
        
        assert_eq!(system.cap_count(), 4);
        
        let (_, rights) = system.get_cap(level3).unwrap();
        assert!(rights.has(Rights::READ));
        assert!(!rights.has(Rights::GRANT));
    }

    #[test]
    fn test_revoke_does_not_cascade() {
        let system = MockCapabilitySystem::new();
        let parent = system.create_cap(1, Rights::new(Rights::READ | Rights::GRANT));
        let child = system.delegate(parent, Rights::new(Rights::READ)).unwrap();
        
        system.revoke(parent);
        
        assert!(system.get_cap(parent).is_none());
        assert!(system.get_cap(child).is_some());
    }
}
