// OGP 이미지(1200x630)를 satori + resvg로 생성합니다.
// 실행: bun run scripts/generate-ogp.ts
// 폰트: scripts/.cache/에 Noto Sans KR(woff)을 배치합니다
import satori from 'satori';
import { Resvg } from '@resvg/resvg-js';
import { readFile, writeFile } from 'node:fs/promises';
import type { ReactElement } from 'react';

const C = {
  bg: '#12141d',
  panel: '#1c1f2c',
  panelBorder: '#2c3044',
  text: '#f2f3f8',
  sub: '#a7aec2',
  dim: '#697089',
  indigo: '#8b93ff',
  indigoDeep: '#4c56d8',
  orange: '#e5a458',
};

// JSX를 사용하지 않고 React 요소와 구조적으로 호환되는 객체를 만듭니다
// (satori 문서의 "Use without JSX" 방식)
const h = (
  type: string,
  style: Record<string, unknown>,
  ...children: (ReactElement | string)[]
): ReactElement => {
  if (children.length > 1 && style['display'] !== 'flex') {
    throw new Error(`display:flex가 없는 다중 자식 노드: ${JSON.stringify(style)}`);
  }
  // children을 빈 배열로 넘기면 satori가 다중 자식 노드로 잘못 판단해
  // display:flex를 요구할 수 있으므로 자식이 없을 때는 키 자체를 생략합니다
  return {
    type,
    props: children.length > 0 ? { style, children } : { style },
    key: null,
  };
};

const root = h(
  'div',
  {
    width: 1200,
    height: 630,
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'center',
    background: '#ffffff',
    color: '#111111',
    fontFamily: 'Noto Sans KR',
    padding: 80,
    gap: 40,
  },
  h(
    'div',
    {
      display: 'flex',
      flexDirection: 'column',
      fontSize: 96,
      fontWeight: 700,
      lineHeight: 1.3,
    },
    h('div', { display: 'flex' }, 'Rust로 시작하는'),
    h(
      'div',
      { display: 'flex' },
      h('div', { display: 'flex', color: '#ff77aa' }, 'CPU'),
      h('div', { display: 'flex' }, '와 '),
      h('div', { display: 'flex', color: '#00a752' }, 'GPU')
    )
  ),
  h(
    'div',
    { display: 'flex', fontSize: 26, color: '#8a8f9e', marginTop: 16 },
    'kyooni18/rust-cpu-gpu-book-kor'
  )
);

async function main(): Promise<void> {
  const [bold, regular] = await Promise.all([
    readFile('scripts/.cache/noto-kr-700.woff'),
    readFile('scripts/.cache/noto-kr-400.woff'),
  ]);

  const svg = await satori(root, {
    width: 1200,
    height: 630,
    fonts: [
      { name: 'Noto Sans KR', data: bold, weight: 700, style: 'normal' },
      { name: 'Noto Sans KR', data: regular, weight: 400, style: 'normal' },
    ],
  });

  const png = new Resvg(svg, { fitTo: { mode: 'width', value: 1200 } })
    .render()
    .asPng();
  await writeFile('public/ogp.png', png);
  console.log(`public/ogp.png (${(png.length / 1024).toFixed(0)}KB)`);
}

main().catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
