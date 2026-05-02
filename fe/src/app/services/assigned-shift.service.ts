import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiService } from './api.service';
import {
  AssignedShift,
  CreateAssignedShiftRequest,
  UpdateAssignedShiftRequest,
  InitAssignedShiftsRequest,
  InitAssignedShiftsResponse
} from '@models/index';

@Injectable({
  providedIn: 'root'
})
export class AssignedShiftService {

  constructor(private apiService: ApiService) { }

  /**
   * Get all assigned shifts
   */
  getAssignedShifts(): Observable<AssignedShift[]> {
    return this.apiService.get('/assigned-shifts');
  }

  /**
   * Get a single assigned shift by ID
   */
  getAssignedShift(id: string): Observable<AssignedShift> {
    return this.apiService.get(`/assigned-shifts/${id}`);
  }

  /**
   * Get assigned shifts for a specific department
   */
  getDepartmentAssignedShifts(departmentId: string): Observable<AssignedShift[]> {
    return this.apiService.get(`/departments/${departmentId}/assigned-shifts`);
  }

  /**
   * Create a new assigned shift
   */
  createAssignedShift(request: CreateAssignedShiftRequest): Observable<AssignedShift> {
    return this.apiService.post('/assigned-shifts', request);
  }

  /**
   * Update an existing assigned shift
   */
  updateAssignedShift(id: string, request: UpdateAssignedShiftRequest): Observable<AssignedShift> {
    return this.apiService.put(`/assigned-shifts/${id}`, request);
  }

  /**
   * Delete an assigned shift
   */
  deleteAssignedShift(id: string): Observable<void> {
    return this.apiService.delete(`/assigned-shifts/${id}`);
  }

  /**
   * Bulk initialize assigned shifts for a date range
   */
  initializeAssignedShifts(request: InitAssignedShiftsRequest): Observable<InitAssignedShiftsResponse> {
    return this.apiService.post('/assigned-shifts/init', request);
  }
}
