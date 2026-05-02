import { Component, OnInit, Inject } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { MatSnackBar } from '@angular/material/snack-bar';
import { EmployeeService } from '@services/employee.service';
import { AssignedShiftService } from '@services/assigned-shift.service';
import { AssignedShift, Employee } from '@models/index';
import { translateShiftType } from '@models/translations';

export interface AssignEmployeesDialogData {
  assignedShift: AssignedShift;
  shiftInfo: any;
  departmentName: string;
  stateColor?: { background: string; border: string; text: string; icon: string };
}

@Component({
  selector: 'app-assign-employees-dialog',
  templateUrl: './assign-employees-dialog.component.html',
  styleUrls: ['./assign-employees-dialog.component.scss']
})
export class AssignEmployeesDialogComponent implements OnInit {
  form: FormGroup;
  employees: Employee[] = [];
  selectedEmployeeIds: Set<string> = new Set();
  initialEmployeeIds: Set<string> = new Set();
  loading = false;
  submitting = false;

  constructor(
    private fb: FormBuilder,
    private employeeService: EmployeeService,
    private assignedShiftService: AssignedShiftService,
    private snackBar: MatSnackBar,
    public dialogRef: MatDialogRef<AssignEmployeesDialogComponent>,
    @Inject(MAT_DIALOG_DATA) public data: AssignEmployeesDialogData
  ) {
    this.form = this.fb.group({
      employees: [[], Validators.required]
    });

    // Initialize selected employees
    if (data.assignedShift.mitarbeiter_ids && data.assignedShift.mitarbeiter_ids.length > 0) {
      this.selectedEmployeeIds = new Set(data.assignedShift.mitarbeiter_ids);
      this.initialEmployeeIds = new Set(data.assignedShift.mitarbeiter_ids);
    }
  }

  ngOnInit(): void {
    this.loadEmployees();
  }

  private loadEmployees(): void {
    this.loading = true;
    this.employeeService.getEmployees().subscribe({
      next: (employees) => {
        this.employees = employees;
      },
      error: (error) => {
        console.error('Failed to load employees', error);
        this.snackBar.open('Fehler beim Laden der Mitarbeiter', 'Schließen', { duration: 5000 });
      },
      complete: () => {
        this.loading = false;
      }
    });
  }

  toggleEmployee(employee: Employee): void {
    if (this.selectedEmployeeIds.has(employee.id)) {
      this.selectedEmployeeIds.delete(employee.id);
    } else {
      this.selectedEmployeeIds.add(employee.id);
    }
  }

  isEmployeeSelected(employeeId: string): boolean {
    return this.selectedEmployeeIds.has(employeeId);
  }

  getSelectedCount(): number {
    return this.selectedEmployeeIds.size;
  }

  onSubmit(): void {
    if (this.selectedEmployeeIds.size === 0) {
      this.snackBar.open('Bitte wählen Sie mindestens einen Mitarbeiter aus', 'Schließen', { duration: 5000 });
      return;
    }

    this.submitting = true;

    const updateRequest = {
      mitarbeiter_ids: Array.from(this.selectedEmployeeIds)
    };

    this.assignedShiftService.updateAssignedShift(this.data.assignedShift.id, updateRequest).subscribe({
      next: () => {
        this.snackBar.open(
          `${this.selectedEmployeeIds.size} Mitarbeiter erfolgreich zugewiesen`,
          'Schließen',
          { duration: 5000 }
        );
        this.dialogRef.close(true);
      },
      error: (error) => {
        console.error('Failed to assign employees', error);
        const errorMessage = this.getErrorMessage(error);
        this.snackBar.open(errorMessage, 'Schließen', { duration: 8000 });
        this.submitting = false;
      }
    });
  }

  onCancel(): void {
    this.dialogRef.close(false);
  }

  selectAll(): void {
    this.employees.forEach(emp => this.selectedEmployeeIds.add(emp.id));
  }

  clearAll(): void {
    this.selectedEmployeeIds.clear();
  }

  // Each employee provides 8 hours of work
  private readonly HOURS_PER_EMPLOYEE = 8;

  calculateCoverageHours(): number {
    return this.selectedEmployeeIds.size * this.HOURS_PER_EMPLOYEE;
  }

  calculateInitialCoverageHours(): number {
    return this.initialEmployeeIds.size * this.HOURS_PER_EMPLOYEE;
  }

  getHoursDifference(): number {
    return this.calculateCoverageHours() - this.calculateInitialCoverageHours();
  }

  isCoverageSatisfied(): boolean {
    const requiredHours = this.data.shiftInfo?.arbeitsstunden || 0;
    return this.calculateCoverageHours() >= requiredHours;
  }

  hasSelectionChanged(): boolean {
    if (this.selectedEmployeeIds.size !== this.initialEmployeeIds.size) {
      return true;
    }
    for (const id of this.selectedEmployeeIds) {
      if (!this.initialEmployeeIds.has(id)) {
        return true;
      }
    }
    return false;
  }

  /**
   * Extract a user-friendly error message from the API error response
   */
  private getErrorMessage(error: any): string {
    const errorBody = error?.error;
    const currentDept = this.data.departmentName;

    // Check for scheduling conflict error
    if (errorBody?.error === 'scheduling_conflict' && errorBody?.details) {
      const details = errorBody.details;
      const employeeName = details.employee_name || 'Mitarbeiter';
      const conflictingShift = details.conflicting_shift;

      if (conflictingShift) {
        const shiftType = translateShiftType(conflictingShift.shift_type);
        const startTime = conflictingShift.start_time || '';
        const endTime = conflictingShift.end_time || '';
        const timeRange = startTime && endTime ? ` (${startTime}-${endTime})` : '';
        const conflictDept = conflictingShift.department_name;
        const conflictDeptInfo = conflictDept ? ` in ${conflictDept}` : '';

        return `${employeeName} kann nicht zur ${currentDept} zugewiesen werden: bereits für ${shiftType}${timeRange}${conflictDeptInfo} eingeplant`;
      }

      return `${employeeName} kann nicht zur ${currentDept} zugewiesen werden: Terminkonflikt`;
    }

    // Use the message field if available
    if (errorBody?.message) {
      return errorBody.message;
    }

    // Fallback to generic error message
    return `Fehler beim Zuweisen der Mitarbeiter zur ${currentDept}`;
  }
}
