import { Component, OnInit } from '@angular/core';
import { DepartmentService } from '@services/department.service';
import { Department, getDepartmentColorById, DepartmentColorScheme } from '@models/index';

@Component({
  selector: 'app-departments',
  templateUrl: './departments.component.html',
  styleUrls: ['./departments.component.scss']
})
export class DepartmentsComponent implements OnInit {
  departments: Department[] = [];
  loading = false;
  error: string | null = null;

  constructor(private departmentService: DepartmentService) { }

  ngOnInit(): void {
    this.loadDepartments();
  }

  loadDepartments(): void {
    this.loading = true;
    this.departmentService.getDepartments().subscribe({
      next: (data) => {
        this.departments = data;
      },
      error: (error) => {
        this.error = 'Fehler beim Laden der Abteilungen';
        console.error(error);
      },
      complete: () => {
        this.loading = false;
      }
    });
  }

  getDepartmentColor(departmentId: string): DepartmentColorScheme {
    return getDepartmentColorById(departmentId);
  }

  onAddDepartment(): void {
    // TODO: Implement add department dialog
  }

  onEditDepartment(department: Department): void {
    // TODO: Implement edit department dialog
  }

  onDeleteDepartment(id: string): void {
    // TODO: Implement delete department with confirmation
  }
}
