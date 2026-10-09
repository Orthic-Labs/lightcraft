const SECTION_ALIASES: Record<string, string> = {
  light: 'Light', color: 'Color', exposure: 'Exposure', curve: 'Tone Curve', tonecurve: 'Tone Curve',
  mixer: 'Color Mixer', colormixer: 'Color Mixer', bwmix: 'B&W Mixer', bwmixer: 'B&W Mixer',
  grading: 'Color Grading', colorgrading: 'Color Grading', effects: 'Effects', vignette: 'Vignette', grain: 'Grain', detail: 'Detail',
  optics: 'Optics', geometry: 'Geometry', profile: 'Profile', calibration: 'Calibration',
  pointcolor: 'Point Color', redeye: 'Red Eye',
};

function sectionToken(value: string): string { return value.trim().toLowerCase().replace(/[^a-z0-9]/g, ''); }
export function canonicalSection(value: string): string { const trimmed = value.trim(); const token = sectionToken(trimmed); return Object.hasOwn(SECTION_ALIASES, token) ? SECTION_ALIASES[token] : (trimmed || 'Light'); }

export function sectionIsOpen(sections: Record<string, boolean>, key: string): boolean {
  const canonical = canonicalSection(key);
  const matches = Object.entries(sections).filter(([stored]) => canonicalSection(stored) === canonical);
  const exact = matches.find(([stored]) => stored === canonical);
  return exact?.[1] ?? matches[0]?.[1] ?? (canonical === 'Light' || canonical === 'Color');
}

export function toggledSection(sections: Record<string, boolean>, key: string): Record<string, boolean> {
  const canonical = canonicalSection(key);
  return { ...sections, [canonical]: !sectionIsOpen(sections, canonical) };
}
