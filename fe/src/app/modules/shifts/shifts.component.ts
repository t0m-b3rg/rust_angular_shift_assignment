import { Component, OnInit } from '@angular/core';
import { ShiftService } from '@services/shift.service';
import { DepartmentService } from '@services/department.service';
import { Shift, Department, getDepartmentColorById, DepartmentColorScheme } from '@models/index';

interface ShiftsByDepartment {
  [departmentId: string]: {
    department: Department | null;
    shifts: Shift[];
  };
}

@Component({
  selector: 'app-shifts',
  templateUrl: './shifts.component.html',
  styleUrls: ['./shifts.component.scss']
})
export class ShiftsComponent implements OnInit {
  shifts: Shift[] = [];
  departments: Department[] = [];
  shiftsByDepartment: ShiftsByDepartment = {};
  loading = false;
  error: string | null = null;

  displayedColumns: string[] = ['typ', 'start_time', 'end_time', 'arbeitsstunden', 'actions'];

  constructor(
    private shiftService: ShiftService,
    private departmentService: DepartmentService
  ) { }

  ngOnInit(): void {
    this.loadData();
  }

  loadData(): void {
    this.loading = true;
    this.departmentService.getDepartments().subscribe({
      next: (departments) => {
        this.departments = departments;
        this.loadShifts();
      },
      error: (error) => {
        this.error = 'Fehler beim Laden der Abteilungen';
        console.error(error);
        this.loading = false;
      }
    });
  }

  loadShifts(): void {
    this.shiftService.getShifts().subscribe({
      next: (data) => {
        this.shifts = data;
        this.groupShiftsByDepartment();
      },
      error: (error) => {
        this.error = 'Fehler beim Laden der Schichten';
        console.error(error);
      },
      complete: () => {
        this.loading = false;
      }
    });
  }

  groupShiftsByDepartment(): void {
    this.shiftsByDepartment = {};
    
    for (const shift of this.shifts) {
      const deptId = shift.department_id || 'unknown';
      if (!this.shiftsByDepartment[deptId]) {
        const dept = this.departments.find(d => d.id === deptId) || null;
        this.shiftsByDepartment[deptId] = {
          department: dept,
          shifts: []
        };
      }
      this.shiftsByDepartment[deptId].shifts.push(shift);
    }
  }

  getDepartmentIds(): string[] {
    return Object.keys(this.shiftsByDepartment);
  }

  getDepartmentName(departmentId: string): string {
    const group = this.shiftsByDepartment[departmentId];
    return group?.department?.name || 'Unbekannt';
  }

  getDepartmentBereich(departmentId: string): string {
    const group = this.shiftsByDepartment[departmentId];
    return group?.department?.bereich || '';
  }

  getDepartmentColor(departmentId: string): DepartmentColorScheme {
    return getDepartmentColorById(departmentId);
  }

  onAddShift(departmentId?: string): void {
    // TODO: Implement add shift dialog with pre-selected department
    console.log('Add shift to department:', departmentId);
  }

  onEditShift(shift: Shift): void {
    // TODO: Implement edit shift dialog
  }

  onDeleteShift(id: string): void {
    // TODO: Implement delete shift with confirmation
  }
}
