# Rust로 시작하는 CPU와 GPU

웹 애플리케이션 개발자를 위한, Rust로 따라가는 CPU·GPU 교과서입니다.
Astro Starlight(MDX)로 만든 문서 사이트입니다.

**📖 원문 공개 사이트: https://rust-cpu-gpu-book.void.app**

- **기초편 12장 + 응용편 16장 + 용어집(약 140개 용어)**으로 이루어진 총 28장 구성입니다
- 본문에 있는 Rust 코드 대부분은 브라우저에서
  [Rust Playground](https://play.rust-lang.org/)를 통해 **바로 실행하고 편집**할 수 있습니다
- 그림은 Mermaid와 인라인 SVG로 작성되어 있습니다. 본문에 제시된 측정값은 모두 실제로
  측정한 값입니다(아래의 "측정 환경"을 참고하세요. 어떤 환경에서 측정했는지는 본문에 표시되어 있습니다)

## 측정 환경

본문의 "저자 실측" 값은 2026년 8월에 다음 두 환경에서 측정했습니다.

| 환경 | 주요 용도 | 세부 정보 |
| --- | --- | --- |
| Rust Playground | 브라우저에서 실행할 수 있는 일반 스니펫 | play.rust-lang.org의 공유 x86-64 Linux 환경(2 vCPU, 메모리 제한 있음). stable release 빌드를 기본으로 하며 debug/nightly를 사용하는 부분은 본문에 표시합니다. 공유 환경이므로 실행할 때마다 수십% 정도 차이가 날 수 있습니다 |
| 로컬 Mac | wgpu/BLAS/criterion 등 로컬 실행 examples | Apple M4(CPU 10코어 = P4+E6, GPU 10코어, 통합 메모리 32GB), macOS 26.3, rustc 1.95.0, wgpu 30.0.0 |

배율이나 경향은 환경에 따라 달라질 수 있습니다(23장에서는 환경 차이 때문에
일반적인 성능 최적화 상식과 다른 결과가 나오는 사례도 다룹니다). 직접 다시 측정해 보는 것을 권합니다.

## 읽기 / 개발

원문은 https://rust-cpu-gpu-book.void.app 에서 바로 읽을 수 있습니다.
로컬에서 실행하려면 다음 명령을 사용합니다.

```sh
bun install
bun run dev      # 개발 서버 (http://localhost:4321)
bun run build    # dist/에 정적 빌드
bun run preview  # 빌드 결과 확인
bun run deploy   # void로 Cloudflare Workers에 배포
```

## 목차

**기초편** — 사전 지식이 없는 상태에서 최단 경로로 "원리를 바탕으로 성능을 설명할 수 있는" 수준까지

| Part | 장 |
| --- | --- |
| I CPU 알아보기 | 1 프로그램은 어떻게 동작하는가 / 2 메모리 계층과 캐시 / 3 파이프라인과 분기 예측 / 4 SIMD와 벡터화 / 5 멀티코어와 병렬 처리 |
| II Rust와 최적화 | 6 컴파일러가 하는 일 / 7 제로 비용 추상화의 실제 / 8 측정한 다음 최적화하기 |
| III GPU 알아보기 | 9 GPU라는 계산 장치 / 10 GPU 메모리와 전송 / 11 Rust에서 GPU 사용하기 / 12 CPU와 GPU 구분해서 사용하기 |

**응용편** — 전체 체계를 완성하기 위한 핵심 주제입니다. 관심 있는 장부터 독립적으로 읽을 수 있습니다

| Part | 장 |
| --- | --- |
| IV CPU와 메모리 심층 | 13 수의 표현 / 14 가상 메모리와 TLB / 15 캐시 내부 구조 / 16 프런트엔드와 top-down 분석 / 17 메모리 모델과 동시성 자료구조 |
| V Rust 심층 | 18 할당자 / 19 async의 실제 구조 / 20 unsafe와 정의되지 않은 동작 및 FFI / 21 빌드 제대로 활용하기 / 22 자료구조의 실제 성능 |
| VI GPU 심층 | 23 커널 최적화 체계 / 24 전송과 실행 겹치기 / 25 행렬 엔진과 혼합 정밀도 / 26 GPU 성능 측정 |
| VII 시스템과 실전 | 27 OS 계층의 비용 / 28 실전 성능 엔지니어링(+지식 지도) |

## 저장소 구조

```
src/content/docs/   # 본문 (MDX)
  cpu/              #   Part I   CPU 알아보기 (1-5장)
  rust-opt/         #   Part II  Rust와 최적화 (6-8장)
  gpu/              #   Part III GPU 알아보기 (9-12장)
  cpu-deep/         #   Part IV  CPU와 메모리 심층 (13-17장)
  rust-deep/        #   Part V   Rust 심층 (18-22장)
  gpu-deep/         #   Part VI  GPU 심층 (23-26장)
  systems/          #   Part VII 시스템과 실전 (27-28장)
  appendix/         #   용어집·더 공부하려면
src/snippets/       # 실행 가능한 Rust 코드 (표시·실행·검증의 단일 소스)
src/components/     # RustPlay.astro (Playground 실행 컴포넌트)
examples/           # 로컬 실행용 Cargo 워크스페이스 (wgpu / criterion / BLAS)
scripts/play.sh     # 스니펫을 Playground API로 실행·검증하는 스크립트
scripts/generate-ogp.ts  # OGP 이미지 생성 (satori + resvg, bun run ogp)
```

## examples 실행

GPU나 OS 라이브러리를 사용하는 장의 코드는 브라우저에서 실행할 수 없으므로
로컬에서 실행합니다(wgpu 계열 예제는 GPU가 있는 환경이 필요합니다).

```sh
cd examples
cargo run --release -p ch11-vector-add   # 11장: wgpu 벡터 덧셈
cargo run --release -p ch12-matmul       # 12장: 행렬 곱 CPU 3가지 방식 + GPU 3가지 방식
cargo run --release -p ch23-reduction    # 23장: GPU reduction 3단계
cargo run --release -p ch24-overlap      # 24장: 동기화 지점 감소로 2.7배
cargo run --release -p ch25-gemm-lib     # 25장: Accelerate(BLAS)와 비교 (macOS 전용)
cargo run --release -p ch26-timestamp    # 26장: GPU 타임스탬프 측정
cargo bench -p ch08-bench                # 8장: criterion 벤치마크
```

## 스니펫 검증

`src/snippets/`의 코드는 화면 표시, 브라우저 실행, 검증에 사용하는 단일 소스입니다.

- 표준 라이브러리만 사용하는 스니펫은 `rustc --edition 2024`로
  그대로 컴파일할 수 있습니다(`nightly-` 접두사가 붙은 파일은 nightly 사용)
- 외부 크레이트나 OS 라이브러리에 의존하는 스니펫(rayon / tokio / libc)은
  Playground 실행 스크립트로 검증합니다

```sh
# 표준 라이브러리만 사용하는 스니펫 컴파일 검증
for f in src/snippets/*/*.rs; do
  rustc --edition 2024 --crate-type bin -O -o /tmp/check "$f" || echo "skip(외부 크레이트 필요): $f"
done

# Playground API에서 실행 (의존 크레이트가 있는 스니펫도 실행 가능)
bash scripts/play.sh src/snippets/ch05/rayon.rs release
```
