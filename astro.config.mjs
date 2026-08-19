// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';

export default defineConfig({
  site: 'https://rust-cpu-gpu-book.void.app',
  integrations: [
    // astro-mermaid는 markdown 처리보다 먼저 등록합니다
    mermaid({ autoTheme: true }),
    starlight({
      title: 'Rust로 시작하는 CPU와 GPU',
      description:
        '웹 애플리케이션 개발자를 위한, Rust로 따라가는 CPU·GPU 교과서',
      head: [
        {
          tag: 'meta',
          attrs: {
            property: 'og:image',
            content: 'https://rust-cpu-gpu-book.void.app/ogp.png',
          },
        },
        { tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
        { tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
        {
          tag: 'meta',
          attrs: { name: 'twitter:card', content: 'summary_large_image' },
        },
        {
          tag: 'meta',
          attrs: {
            name: 'twitter:image',
            content: 'https://rust-cpu-gpu-book.void.app/ogp.png',
          },
        },
      ],
      defaultLocale: 'root',
      locales: {
        root: { label: '한국어', lang: 'ko' },
      },
      customCss: ['./src/styles/custom.css'],
      tableOfContents: { minHeadingLevel: 2, maxHeadingLevel: 3 },
      sidebar: [
        { label: '들어가며', link: '/' },
        {
          label: 'Part I — CPU 이해하기',
          items: [{ autogenerate: { directory: 'cpu' } }],
        },
        {
          label: 'Part II — Rust와 최적화',
          items: [{ autogenerate: { directory: 'rust-opt' } }],
        },
        {
          label: 'Part III — GPU 이해하기',
          items: [{ autogenerate: { directory: 'gpu' } }],
        },
        {
          label: 'Part IV — CPU와 메모리의 심층',
          items: [{ autogenerate: { directory: 'cpu-deep' } }],
        },
        {
          label: 'Part V — Rust의 심층',
          items: [{ autogenerate: { directory: 'rust-deep' } }],
        },
        {
          label: 'Part VI — GPU의 심층',
          items: [{ autogenerate: { directory: 'gpu-deep' } }],
        },
        {
          label: 'Part VII — 시스템과 실전',
          items: [{ autogenerate: { directory: 'systems' } }],
        },
        { label: '부록', items: [{ autogenerate: { directory: 'appendix' } }] },
      ],
    }),
  ],
});
