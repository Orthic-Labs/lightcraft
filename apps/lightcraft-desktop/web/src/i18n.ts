import zhHans from '../../../../crates/ui-egui/locales/zh-hans.json';
import zhHant from '../../../../crates/ui-egui/locales/zh-hant.json';
import ja from '../../../../crates/ui-egui/locales/ja.json';

export const LOCALES = ['en', 'zh-hans', 'zh-hant', 'ja'] as const;
export type LocaleCode = (typeof LOCALES)[number];
type Catalog = Record<string, string>;

const catalogs: Record<Exclude<LocaleCode, 'en'>, Catalog> = {
  'zh-hans': zhHans as Catalog,
  'zh-hant': zhHant as Catalog,
  ja: ja as Catalog,
};

/** Normalize persisted BCP-47-ish tags to baseline egui catalog codes. */
export function normalizeLocale(value: unknown): LocaleCode {
  const raw = typeof value === 'string' ? value.trim().toLowerCase().replace(/_/g, '-') : '';
  const tag = raw.split(/[.@]/, 1)[0];
  if (tag === 'ja' || tag.startsWith('ja-')) return 'ja';
  if (tag === 'zh-hant' || tag === 'zh-tw' || tag === 'zh-hk' || tag === 'zh-mo' || tag.startsWith('zh-hant-')) return 'zh-hant';
  if (tag === 'zh' || tag === 'zh-hans' || tag === 'zh-cn' || tag === 'zh-sg' || tag.startsWith('zh-hans-')) return 'zh-hans';
  return 'en';
}

/** Translate built-in source strings; user content and unknown strings fall back unchanged. */
export function translate(locale: unknown, source: string): string {
  const code = normalizeLocale(locale);
  return code === 'en' ? source : catalogs[code][source] ?? source;
}

