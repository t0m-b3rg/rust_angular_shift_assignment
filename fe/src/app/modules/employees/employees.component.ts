import { Component, OnInit } from '@angular/core';
import { EmployeeService } from '@services/employee.service';
import { Employee } from '@models/index';

@Component({
  selector: 'app-employees',
  templateUrl: './employees.component.html',
  styleUrls: ['./employees.component.scss']
})
export class EmployeesComponent implements OnInit {
  employees: Employee[] = [];
  loading = false;
  error: string | null = null;

  displayedColumns: string[] = ['name', 'rolle', 'qualifikation', 'actions'];

  constructor(private employeeService: EmployeeService) { }

  ngOnInit(): void {
    this.loadEmployees();
  }

  loadEmployees(): void {
    this.loading = true;
    this.employeeService.getEmployees().subscribe({
      next: (data) => {
        this.employees = data;
      },
      error: (error) => {
        this.error = 'Failed to load employees';
        console.error(error);
      },
      complete: () => {
        this.loading = false;
      }
    });
  }

  onAddEmployee(): void {
    // TODO: Implement add employee dialog
  }

  onEditEmployee(employee: Employee): void {
    // TODO: Implement edit employee dialog
  }

  onDeleteEmployee(id: string): void {
    // TODO: Implement delete employee with confirmation
  }
}
