use loom_core::opfs::OpfsCoordinator;
use loom_storage::abi::{ContainerHeader, PageType, HEADER_SIZE};
use loom_storage::container::LoomContainer;
use loom_storage::pager::Pager;
use std::collections::HashMap;

#[test]
fn test_binary_container_abi_seqlock_and_paging() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("integ_test_abi_{}.loom", ts));

    let mut container = LoomContainer::open_or_create(&path).unwrap();
    assert!(container.save().is_ok());

    let stats = container.stats();
    assert_eq!(stats.page_size, 4096);
    assert!(stats.total_pages >= 2);
    assert_eq!(stats.seqlock_version % 2, 0); // Must be even (stable reader view)

    // Verify raw header bytes directly from file
    let mut file = std::fs::File::open(&path).unwrap();
    use std::io::Read;
    let mut header_buf = [0u8; HEADER_SIZE];
    file.read_exact(&mut header_buf).unwrap();

    let header = ContainerHeader::deserialize(&header_buf).unwrap();
    assert_eq!(&header.magic, b"LOOMDB\x01\x00");
    assert!(header.is_reader_view_valid());

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_deep_cow_branch_page_isolation() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("integ_cow_{}.loom", ts));

    let mut pager = Pager::open_or_create(&path).unwrap();
    let mut main_map = HashMap::new();

    // Allocate 3 logical pages for main
    for log_id in 1..=3 {
        let data = format!("main_page_{}_v1", log_id);
        pager.write_page_cow(log_id, &mut main_map, data.as_bytes(), PageType::BTreeLeaf).unwrap();
    }

    // Fork branch 'audit'
    let mut audit_map = pager.register_branch_cow(&main_map);

    // Modify only page 2 in audit branch (COW allocates new physical slot)
    let new_phys = pager.write_page_cow(2, &mut audit_map, b"audit_page_2_modified", PageType::BTreeLeaf).unwrap();
    assert_ne!(new_phys, *main_map.get(&2).unwrap());

    // Verify main page 2 is intact
    let main_p2 = pager.read_page(*main_map.get(&2).unwrap()).unwrap();
    let audit_p2 = pager.read_page(*audit_map.get(&2).unwrap()).unwrap();

    assert!(main_p2[32..].starts_with(b"main_page_2_v1"));
    assert!(audit_p2[32..].starts_with(b"audit_page_2_modified"));

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_opfs_multi_tab_coordination_and_cdc() {
    let mut coordinator = OpfsCoordinator::new("analytics");

    // Tab 1 acquires lease
    let lease1 = coordinator.acquire_writer_lease("tab_chrome_1", "worker_main", 1000, 3000);
    assert!(lease1.is_ok());

    // Tab 2 fails to acquire while lease 1 is active
    let lease2_fail = coordinator.acquire_writer_lease("tab_firefox_2", "worker_reader", 1500, 3000);
    assert!(lease2_fail.is_err());

    // Tab 1 emits CDC event on table mutation
    let cdc = coordinator.create_cdc_event("tab_chrome_1", 42, vec!["sensor_data".into()], 2000);
    assert_eq!(cdc.mutated_tables[0], "sensor_data");

    // Tab 1 releases lease
    assert!(coordinator.release_writer_lease("tab_chrome_1").is_ok());

    // Tab 2 now succeeds
    let lease2_ok = coordinator.acquire_writer_lease("tab_firefox_2", "worker_reader", 2500, 3000);
    assert!(lease2_ok.is_ok());
}
