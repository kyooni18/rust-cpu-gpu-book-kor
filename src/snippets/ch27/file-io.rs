use std::fs;
use std::time::Instant;

fn main() {
    // 64MB 파일을 준비합니다(프로세스 ID를 붙인 임시 파일 이름을 사용합니다)
    let path = std::env::temp_dir().join(format!("book_io_{}.bin", std::process::id()));
    let path = path.as_path();
    let size = 64 * 1024 * 1024usize;
    let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    let start = Instant::now();
    fs::write(path, &data).unwrap();
    println!("쓰기(64MB)       : {:>9.3?}", start.elapsed());
    drop(data);

    // (1) fs::read로 전체를 읽습니다(커널→사용자 공간 복사)
    for round in 1..=2 {
        let start = Instant::now();
        let buf = fs::read(path).unwrap();
        let sum: u64 = buf.iter().map(|&b| b as u64).sum();
        println!(
            "fs::read {round}회차  : {:>9.3?} (sum={sum})",
            start.elapsed()
        );
    }

    // (2) mmap으로 주소 공간에 매핑해 읽습니다(별도 복사 없음, 14장의 실전 예)
    use std::os::fd::AsRawFd;
    let file = fs::File::open(path).unwrap();
    assert_eq!(file.metadata().unwrap().len() as usize, size);
    let start = Instant::now();
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        )
    };
    assert!(ptr != libc::MAP_FAILED);
    // SAFETY: mmap이 성공했고 size 바이트를 읽을 수 있습니다
    let mapped: &[u8] = unsafe { std::slice::from_raw_parts(ptr as *const u8, size) };
    let sum: u64 = mapped.iter().map(|&b| b as u64).sum();
    println!("mmap + 순회       : {:>9.3?} (sum={sum})", start.elapsed());
    let rc = unsafe { libc::munmap(ptr, size) };
    assert_eq!(rc, 0);
    fs::remove_file(path).unwrap();
}
