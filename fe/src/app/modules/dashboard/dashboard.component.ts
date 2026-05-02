import { Component, OnInit } from '@angular/core';
import { EmployeeService } from '@services/employee.service';
import { DepartmentService } from '@services/department.service';
import { ShiftService } from '@services/shift.service';
import { AssignedShiftService } from '@services/assigned-shift.service';

@Component({
  selector: 'app-dashboard',
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.scss']
})
export class DashboardComponent implements OnInit {
  employeeCount = 0;
  departmentCount = 0;
  shiftCount = 0;
  assignedShiftCount = 0;
  loading = true;
  error: string | null = null;

  constructor(
    private employeeService: EmployeeService,
    private departmentService: DepartmentService,
    private shiftService: ShiftService,
    private assignedShiftService: AssignedShiftService
  ) { }

  ngOnInit(): void {
    this.loadDashboardData();
  }

  private loadDashboardData(): void {
    this.loading = true;

    // Load all data in parallel
    Promise.all([
      this.employeeService.getEmployees().toPromise(),
      this.departmentService.getDepartments().toPromise(),
      this.shiftService.getShifts().toPromise(),
      this.assignedShiftService.getAssignedShifts().toPromise()
    ])
      .then(([employees, departments, shifts, assignedShifts]) => {
        this.employeeCount = employees?.length || 0;
        this.departmentCount = departments?.length || 0;
        this.shiftCount = shifts?.length || 0;
        this.assignedShiftCount = assignedShifts?.length || 0;
      })
      .catch(error => {
        this.error = 'Failed to load dashboard data';
        console.error(error);
      })
      .finally(() => {
        this.loading = false;
      });
  }
}
