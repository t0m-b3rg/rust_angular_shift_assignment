import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiService } from './api.service';
import { Shift, CreateShiftRequest, UpdateShiftRequest } from '@models/index';

@Injectable({
  providedIn: 'root'
})
export class ShiftService {

  constructor(private apiService: ApiService) { }

  /**
   * Get all shift templates
   */
  getShifts(): Observable<Shift[]> {
    return this.apiService.get('/shifts');
  }

  /**
   * Get a single shift template by ID
   */
  getShift(id: string): Observable<Shift> {
    return this.apiService.get(`/shifts/${id}`);
  }

  /**
   * Get shifts for a specific department
   */
  getDepartmentShifts(departmentId: string): Observable<Shift[]> {
    return this.apiService.get(`/departments/${departmentId}/shifts`);
  }

  /**
   * Create a new shift template
   */
  createShift(request: CreateShiftRequest): Observable<Shift> {
    return this.apiService.post('/shifts', request);
  }

  /**
   * Update an existing shift template
   */
  updateShift(id: string, request: UpdateShiftRequest): Observable<Shift> {
    return this.apiService.put(`/shifts/${id}`, request);
  }

  /**
   * Delete a shift template
   */
  deleteShift(id: string): Observable<void> {
    return this.apiService.delete(`/shifts/${id}`);
  }
}
