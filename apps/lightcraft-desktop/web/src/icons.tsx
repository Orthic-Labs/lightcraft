import type { SVGProps } from 'react';

export type IconName = 'library' | 'develop' | 'compare' | 'survey' | 'people' | 'search' | 'settings' | 'import' | 'export' | 'chevron';

const paths: Record<IconName, string> = {
  library: 'M4 5.5A1.5 1.5 0 0 1 5.5 4h13A1.5 1.5 0 0 1 20 5.5v13A1.5 1.5 0 0 1 18.5 20h-13A1.5 1.5 0 0 1 4 18.5zM8 4v16M4 9h16M13 4v5M13 14v6',
  develop: 'M4 19h16M6 16V8m4 8V5m4 11v-4m4 4V7',
  compare: 'M5 5h6v6H5zM13 13h6v6h-6zM13 5h6v6h-6zM5 13h6v6H5z',
  survey: 'M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z',
  people: 'M8 11a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm8-1a2.5 2.5 0 1 0 0-5M3.5 19a4.5 4.5 0 0 1 9 0M14 14a4 4 0 0 1 6.5 3',
  search: 'm20 20-4.2-4.2M10.8 17a6.2 6.2 0 1 1 0-12.4 6.2 6.2 0 0 1 0 12.4Z',
  settings: 'M12 15.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Zm0-12v2.1m0 13.4v2.1M4.6 4.6l1.5 1.5m11.8 11.8 1.5 1.5M2.9 12H5m14 0h2.1M4.6 19.4l1.5-1.5m11.8-11.8 1.5-1.5',
  import: 'M12 4v11m0 0 4-4m-4 4-4-4M5 19h14',
  export: 'M12 20V9m0 0 4 4m-4-4-4 4M5 5h14',
  chevron: 'm7 10 5 5 5-5',
};

export function Icon({ name, size = 18, strokeWidth = 1.7, ...props }: SVGProps<SVGSVGElement> & { name: IconName; size?: number; strokeWidth?: number }) {
  return <svg {...props} width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name]} /></svg>;
}
