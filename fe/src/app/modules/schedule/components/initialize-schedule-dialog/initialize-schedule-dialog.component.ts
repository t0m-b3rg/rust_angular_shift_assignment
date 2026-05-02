import { Component, Inject } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { MatSnackBar } from '@angular/material/snack-bar';
import { AssignedShiftService } from '@services/assigned-shift.service';
import { DepartmentService } from '@services/department.service';
import { Department, InitAssignedShiftsRequest } from '@models/index';

@Component({
  selector: 'app-initialize-schedule-dialog',
  templateUrl: './initialize-schedule-dialog.component.html',
  styleUrls: ['./initialize-schedule-dialog.component.scss']
})
export class InitializeScheduleDialogComponent {
  form: FormGroup;
  departments: Department[] = [];
  loading = false;
  submitting = false;

  constructor(
    private fb: FormBuilder,
    private assignedShiftService: AssignedShiftService,
    private departmentService: DepartmentService,
    private snackBar: MatSnackBar,
    public dialogRef: MatDialogRef<InitializeScheduleDialogComponent>,
    @Inject(MAT_DIALOG_DATA) public data: any
  ) {
    this.form = this.fb.group({
      start_date: ['', Validators.required],
      end_date: ['', Validators.required],
      department_id: ['']
    });

    this.loadDepartments();
  }

  private loadDepartments(): void {
    this.loading = true;
    this.departmentService.getDepartments().subscribe({
      next: (data) => {
        this.departments = data;
      },
      error: (error) => {
        console.error('Failed to load departments', error);
        this.snackBar.open('Fehler beim Laden der Abteilungen', 'Schließen', { duration: 5000 });
      },
      complete: () => {
        this.loading = false;
      }
    });
  }

  onSubmit(): void {
    if (this.form.invalid) {
      return;
    }

    this.submitting = true;
    
    // Format dates to YYYY-MM-DD
    const startDate = this.formatDateToString(this.form.value.start_date);
    const endDate = this.formatDateToString(this.form.value.end_date);

    const request: InitAssignedShiftsRequest = {
      start_date: startDate,
      end_date: endDate,
      department_id: this.form.value.department_id || undefined
    };

    this.assignedShiftService.initializeAssignedShifts(request).subscribe({
      next: (response) => {
        this.snackBar.open(
          `${response.created} Schichten erfolgreich erstellt`,
          'Schließen',
          { duration: 5000 }
        );
        this.dialogRef.close(true);
      },
      error: (error) => {
        console.error('Failed to initialize schedule', error);
        const dept = this.departments.find(d => d.id === this.form.value.department_id);
        const deptInfo = dept ? ` für ${dept.name}` : '';
        this.snackBar.open(`Fehler beim Initialisieren des Dienstplans${deptInfo}`, 'Schließen', { duration: 5000 });
      },
      complete: () => {
        this.submitting = false;
      }
    });
  }

  private formatDateToString(date: any): string {
    if (!date) {
      return '';
    }

    // If it's already a string, return as-is
    if (typeof date === 'string') {
      return date;
    }

    // If it's a Date object
    if (date instanceof Date) {
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, '0');
      const day = String(date.getDate()).padStart(2, '0');
      return `${year}-${month}-${day}`;
    }

    // If it's an object with year, month, day (from mat-datepicker)
    if (date.year && date.month && date.day) {
      const year = date.year;
      const month = String(date.month).padStart(2, '0');
      const day = String(date.day).padStart(2, '0');
      return `${year}-${month}-${day}`;
    }

    return '';
  }

  onCancel(): void {
    this.dialogRef.close(false);
  }
}
