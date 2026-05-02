import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiService } from './api.service';
import { Employee, CreateEmployeeRequest, UpdateEmployeeRequest } from '@models/index';

@Injectable({
  providedIn: 'root'
})
export class EmployeeService {

  constructor(private apiService: ApiService) { }

  /**
   * Get all employees
   */
  getEmployees(): Observable<Employee[]> {
    return this.apiService.get('/employees');
  }

  /**
   * Get a single employee by ID
   */
  getEmployee(id: string): Observable<Employee> {
    return this.apiService.get(`/employees/${id}`);
  }

  /**
   * Create a new employee
   */
  createEmployee(request: CreateEmployeeRequest): Observable<Employee> {
    return this.apiService.post('/employees', request);
  }

  /**
   * Update an existing employee
   */
  updateEmployee(id: string, request: UpdateEmployeeRequest): Observable<Employee> {
    return this.apiService.put(`/employees/${id}`, request);
  }

  /**
   * Delete an employee
   */
  deleteEmployee(id: string): Observable<void> {
    return this.apiService.delete(`/employees/${id}`);
  }
}
