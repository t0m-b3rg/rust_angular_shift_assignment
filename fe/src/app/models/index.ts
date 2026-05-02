// Employee qualifications
export enum Qualifikation {
  PFLEGEFACHKRAFT = 'Pflegefachkraft',
  ASSISTENZKRAFT = 'Assistenzkraft',
  PFLEGEHELFER = 'Pflegehelfer'
}

// Shift types
export enum SchichtType {
  FRÜH = 'Früh',
  MITTEL = 'Mittel',
  SPÄT = 'Spät',
  NACHT = 'Nacht'
}

// Assigned shift states
export enum AssignedShiftState {
  VALID = 'valid',
  UNASSIGNED = 'unassigned',
  WORK_HOURS_NOT_SATISFIED = 'work_hours_not_satisfied'
}

// Department color interface
export interface DepartmentColorScheme {
  background: string;
  border: string;
  text: string;
}

// Department colors palette
export const DEPARTMENT_COLORS: DepartmentColorScheme[] = [
  { background: '#e3f2fd', border: '#1976d2', text: '#0d47a1' },      // Blue
  { background: '#f3e5f5', border: '#7b1fa2', text: '#4a148c' },      // Purple
  { background: '#e8f5e9', border: '#388e3c', text: '#1b5e20' },      // Green
  { background: '#fff3e0', border: '#f57c00', text: '#e65100' },      // Orange
  { background: '#fce4ec', border: '#c2185b', text: '#880e4f' },      // Pink
  { background: '#e0f2f1', border: '#00796b', text: '#004d40' },      // Teal
  { background: '#f1f8e9', border: '#689f38', text: '#33691e' },      // Light Green
  { background: '#ede7f6', border: '#512da8', text: '#311b92' },      // Deep Purple
];

// Utility function to get department color by ID
export function getDepartmentColorById(departmentId: string | undefined | null): DepartmentColorScheme {
  if (!departmentId) {
    return { background: '#e0e0e0', border: '#757575', text: '#424242' };
  }
  const hash = departmentId.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
  const colorIndex = hash % DEPARTMENT_COLORS.length;
  return DEPARTMENT_COLORS[colorIndex];
}

// Employee model
export interface Employee {
  id: string;
  name: string;
  rolle: string;
  qualifikation: Qualifikation;
  created_at: string;
}

// Department model
export interface Department {
  id: string;
  name: string;
  bereich: string;
  created_at: string;
}

// Shift template model
export interface Shift {
  id: string;
  typ: SchichtType;
  start_time: string;
  end_time: string;
  department_id: string;
  arbeitsstunden: number;
  created_at: string;
}

// Assigned shift model
export interface AssignedShift {
  id: string;
  datum: string; // YYYY-MM-DD
  arbeitsschichten_id: string;
  mitarbeiter_ids: string[];
  state: AssignedShiftState;
  created_at: string;
}

// Request DTOs
export interface CreateEmployeeRequest {
  name: string;
  rolle: string;
  qualifikation: Qualifikation;
}

export interface UpdateEmployeeRequest {
  name?: string;
  rolle?: string;
  qualifikation?: Qualifikation;
}

export interface CreateDepartmentRequest {
  name: string;
  bereich: string;
}

export interface UpdateDepartmentRequest {
  name?: string;
  bereich?: string;
}

export interface CreateShiftRequest {
  typ: SchichtType;
  start_time: string;
  end_time: string;
  department_id: string;
  arbeitsstunden: number;
}

export interface UpdateShiftRequest {
  typ?: SchichtType;
  start_time?: string;
  end_time?: string;
  department_id?: string;
  arbeitsstunden?: number;
}

export interface CreateAssignedShiftRequest {
  datum: string; // YYYY-MM-DD
  arbeitsschichten_id: string;
  mitarbeiter_ids?: string[];
}

export interface UpdateAssignedShiftRequest {
  datum?: string;
  arbeitsschichten_id?: string;
  mitarbeiter_ids?: string[];
}

export interface InitAssignedShiftsRequest {
  start_date: string; // YYYY-MM-DD
  end_date: string; // YYYY-MM-DD
  department_id?: string;
}

export interface InitAssignedShiftsResponse {
  created: number;
  start_date: string;
  end_date: string;
  message: string;
}

// Response wrappers
export interface ApiResponse<T> {
  data: T;
  message?: string;
  success: boolean;
}

export interface ListResponse<T> {
  items: T[];
  total: number;
}
