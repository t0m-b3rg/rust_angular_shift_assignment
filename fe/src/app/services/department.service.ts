import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiService } from './api.service';
import { Department, CreateDepartmentRequest, UpdateDepartmentRequest } from '@models/index';

@Injectable({
  providedIn: 'root'
})
export class DepartmentService {

  constructor(private apiService: ApiService) { }

  /**
   * Get all departments
   */
  getDepartments(): Observable<Department[]> {
    return this.apiService.get('/departments');
  }

  /**
   * Get a single department by ID
   */
  getDepartment(id: string): Observable<Department> {
    return this.apiService.get(`/departments/${id}`);
  }

  /**
   * Create a new department
   */
  createDepartment(request: CreateDepartmentRequest): Observable<Department> {
    return this.apiService.post('/departments', request);
  }

  /**
   * Update an existing department
   */
  updateDepartment(id: string, request: UpdateDepartmentRequest): Observable<Department> {
    return this.apiService.put(`/departments/${id}`, request);
  }

  /**
   * Delete a department
   */
  deleteDepartment(id: string): Observable<void> {
    return this.apiService.delete(`/departments/${id}`);
  }
}
