/**
 * Centralized translations for the application
 */

// Shift type translations
export const SHIFT_TYPE_TRANSLATIONS: Record<string, string> = {
  'früh': 'Frühschicht',
  'mittel': 'Mittelschicht',
  'spät': 'Spätschicht',
  'nacht': 'Nachtschicht',
  // English variants
  'early': 'Frühschicht',
  'middle': 'Mittelschicht',
  'late': 'Spätschicht',
  'night': 'Nachtschicht',
};

// Assigned shift state translations
export const SHIFT_STATE_TRANSLATIONS: Record<string, string> = {
  'valid': 'Gültig',
  'unassigned': 'Nicht zugewiesen',
  'work_hours_not_satisfied': 'Arbeitsstunden nicht erfüllt',
};

// Error type translations
export const ERROR_TYPE_TRANSLATIONS: Record<string, string> = {
  'scheduling_conflict': 'Terminkonflikt',
  'invalid_employee': 'Ungültiger Mitarbeiter',
  'shift_not_found': 'Schicht nicht gefunden',
  'department_not_found': 'Abteilung nicht gefunden',
};

/**
 * Translate a shift type to German
 */
export function translateShiftType(shiftType: string | undefined | null): string {
  if (!shiftType) return 'Schicht';
  const key = shiftType.toLowerCase();
  return SHIFT_TYPE_TRANSLATIONS[key] || `${shiftType}-Schicht`;
}

/**
 * Translate an assigned shift state to German
 */
export function translateShiftState(state: string | undefined | null): string {
  if (!state) return 'Unbekannt';
  return SHIFT_STATE_TRANSLATIONS[state.toLowerCase()] || state;
}

/**
 * Translate an error type to German
 */
export function translateErrorType(errorType: string | undefined | null): string {
  if (!errorType) return 'Fehler';
  return ERROR_TYPE_TRANSLATIONS[errorType.toLowerCase()] || errorType;
}
