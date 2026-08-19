use std::hint::black_box;
use std::time::Instant;

struct BoxNode {
    value: u64,
    next: Option<Box<BoxNode>>,
}

struct ArenaNode {
    value: u64,
    next: u32, // 아레나 내부 인덱스. u32::MAX를 끝 표시로 사용합니다
}

fn main() {
    let n = 1_000_000u32;

    // (1) 노드마다 힙 할당하는 연결 리스트
    let start = Instant::now();
    let mut head: Option<Box<BoxNode>> = None;
    for i in 0..n {
        head = Some(Box::new(BoxNode { value: i as u64, next: head.take() }));
    }
    println!("Box    생성: {:>9.3?}", start.elapsed());

    let start = Instant::now();
    let mut sum = 0u64;
    let mut cur = head.as_deref();
    while let Some(node) = cur {
        sum = sum.wrapping_add(node.value);
        cur = node.next.as_deref();
    }
    println!("Box    순회: {:>9.3?} (sum={sum})", start.elapsed());

    // 재귀 drop으로 스택 오버플로가 나지 않도록 직접 해체하면서 측정합니다
    let start = Instant::now();
    let mut cur = head;
    while let Some(mut node) = cur {
        cur = node.next.take();
    }
    println!("Box    해제: {:>9.3?}", start.elapsed());

    // (2) 아레나(하나의 Vec)에 모아서 저장하는 연결 리스트
    let start = Instant::now();
    let mut arena: Vec<ArenaNode> = Vec::with_capacity(n as usize);
    let mut head = u32::MAX;
    for i in 0..n {
        arena.push(ArenaNode { value: i as u64, next: head });
        head = i;
    }
    println!("아레나 생성: {:>9.3?}", start.elapsed());

    let start = Instant::now();
    let mut sum = 0u64;
    let mut cur = head;
    while cur != u32::MAX {
        let node = &arena[cur as usize];
        sum = sum.wrapping_add(node.value);
        cur = node.next;
    }
    println!("아레나 순회: {:>9.3?} (sum={sum})", start.elapsed());

    let start = Instant::now();
    drop(arena);
    black_box(());
    println!("아레나 해제: {:>9.3?}", start.elapsed());
}
