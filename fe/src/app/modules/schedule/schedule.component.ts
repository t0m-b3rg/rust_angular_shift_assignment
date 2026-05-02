import { Component, OnInit } from '@angular/core';
import { MatDialog } from '@angular/material/dialog';
import { AssignedShiftService } from '@services/assigned-shift.service';
import { ShiftService } from '@services/shift.service';
import { DepartmentService } from '@services/department.service';
import { AssignedShift, Shift, Department, getDepartmentColorById, DepartmentColorScheme } from '@models/index';
import { translateShiftState } from '@models/translations';
import { InitializeScheduleDialogComponent } from './components/initialize-schedule-dialog/initialize-schedule-dialog.component';
import { AssignEmployeesDialogComponent } from './components/assign-employees-dialog/assign-employees-dialog.component';

interface CalendarDay {
  date: Date;
  dateString: string;
  isCurrentMonth: boolean;
  shifts: AssignedShift[];
}

@Component({
  selector: 'app-schedule',
  templateUrl: './schedule.component.html',
  styleUrls: ['./schedule.component.scss']
})
export class ScheduleComponent implements OnInit {
  assignedShifts: AssignedShift[] = [];
  shifts: Shift[] = [];
  departments: Department[] = [];
  loading = false;
  error: string | null = null;

  // Calendar properties
  currentDate: Date = new Date();
  calendarDays: CalendarDay[] = [];
  weekDays: string[] = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  viewMode: 'table' | 'calendar' = 'calendar';
  displayedColumns: string[] = ['datum', 'department', 'typ', 'start_time', 'mitarbeiter_count', 'state', 'actions'];

  constructor(
    private assignedShiftService: AssignedShiftService,
    private shiftService: ShiftService,
    private departmentService: DepartmentService,
    private dialog: MatDialog
  ) { }

  ngOnInit(): void {
    this.loadData();
  }

  private loadData(): void {
    this.loading = true;
    Promise.all([
      this.assignedShiftService.getAssignedShifts().toPromise(),
      this.shiftService.getShifts().toPromise(),
      this.departmentService.getDepartments().toPromise()
    ])
      .then(([assignedShifts, shifts, departments]) => {
        this.assignedShifts = assignedShifts || [];
        this.shifts = shifts || [];
        this.departments = departments || [];
        this.generateCalendar();
      })
      .catch(error => {
        this.error = 'Failed to load schedule data';
        console.error(error);
      })
      .finally(() => {
        this.loading = false;
      });
  }

  private generateCalendar(): void {
    const year = this.currentDate.getFullYear();
    const month = this.currentDate.getMonth();

    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    const startDate = new Date(firstDay);
    startDate.setDate(startDate.getDate() - firstDay.getDay());

    this.calendarDays = [];
    const current = new Date(startDate);

    while (current <= lastDay || current.getDay() !== 0) {
      const dateString = current.toISOString().split('T')[0];
      const isCurrentMonth = current.getMonth() === month;
      const dayShifts = this.assignedShifts.filter(s => s.datum === dateString);

      this.calendarDays.push({
        date: new Date(current),
        dateString,
        isCurrentMonth,
        shifts: dayShifts
      });

      current.setDate(current.getDate() + 1);
    }
  }

  previousMonth(): void {
    this.currentDate.setMonth(this.currentDate.getMonth() - 1);
    this.generateCalendar();
  }

  nextMonth(): void {
    this.currentDate.setMonth(this.currentDate.getMonth() + 1);
    this.generateCalendar();
  }

  getMonthYear(): string {
    const options: Intl.DateTimeFormatOptions = { year: 'numeric', month: 'long' };
    return this.currentDate.toLocaleDateString('de-DE', options);
  }

  getShiftInfo(shiftId: string): Shift | undefined {
    return this.shifts.find(s => s.id === shiftId);
  }

  getDepartmentName(departmentId: string | undefined): string {
    if (!departmentId) return 'Unbekannt';
    const dept = this.departments.find(d => d.id === departmentId);
    return dept ? dept.name : 'Unbekannt';
  }

  getDepartmentColor(departmentId: string | undefined): DepartmentColorScheme {
    return getDepartmentColorById(departmentId);
  }

  getShiftColor(shift: AssignedShift): DepartmentColorScheme {
    const shiftInfo = this.getShiftInfo(shift.arbeitsschichten_id);
    const departmentId = shiftInfo?.department_id;
    return this.getDepartmentColor(departmentId);
  }

  getShiftTypeIcon(shiftType: string | undefined): string {
    if (!shiftType) return 'schedule';
    switch (shiftType.toLowerCase()) {
      case 'früh':
        return 'light_mode';
      case 'mittel':
        return 'wb_sunny';
      case 'spät':
        return 'wb_twilight';
      case 'nacht':
        return 'dark_mode';
      default:
        return 'schedule';
    }
  }

  getStateColor(state: string): { background: string; border: string; text: string; icon: string } {
    switch (state.toLowerCase()) {
      case 'valid':
        return { background: '#c8e6c9', border: '#388e3c', text: '#1b5e20', icon: 'check_circle' };
      case 'unassigned':
        return { background: '#f5f5f5', border: '#757575', text: '#424242', icon: 'assignment_ind' };
      case 'work_hours_not_satisfied':
        return { background: '#fff9c4', border: '#f57f17', text: '#f57f17', icon: 'warning' };
      default:
        return { background: '#eeeeee', border: '#9e9e9e', text: '#616161', icon: 'help' };
    }
  }

  getStateLabel(state: string): string {
    return translateShiftState(state);
  }

  getShiftsByDepartment(dayShifts: AssignedShift[]): { [key: string]: { deptId: string; deptName: string; shifts: AssignedShift[] } } {
    const grouped: { [key: string]: { deptId: string; deptName: string; shifts: AssignedShift[] } } = {};
    
    dayShifts.forEach(shift => {
      const shiftInfo = this.getShiftInfo(shift.arbeitsschichten_id);
      const deptId = shiftInfo?.department_id || 'unknown';
      const deptName = this.getDepartmentName(deptId);
      
      if (!grouped[deptId]) {
        grouped[deptId] = { deptId, deptName, shifts: [] };
      }
      grouped[deptId].shifts.push(shift);
    });
    
    return grouped;
  }

  getDepartmentIds(grouped: { [key: string]: any }): string[] {
    return Object.keys(grouped);
  }

  onInitializeSchedule(): void {
    const dialogRef = this.dialog.open(InitializeScheduleDialogComponent, {
      width: '600px',
      maxWidth: '90vw',
      height: 'auto',
      maxHeight: '90vh',
      disableClose: true,
      hasBackdrop: true,
      backdropClass: 'initialize-schedule-backdrop',
      panelClass: 'initialize-schedule-panel',
      autoFocus: 'first-tabbable',
      restoreFocus: true,
      enterAnimationDuration: 200,
      exitAnimationDuration: 200
    });

    dialogRef.afterClosed().subscribe(result => {
      if (result) {
        this.loadData();
      }
    });
  }

  toggleViewMode(): void {
    this.viewMode = this.viewMode === 'table' ? 'calendar' : 'table';
  }

  onShiftClick(shift: AssignedShift): void {
    const shiftInfo = this.getShiftInfo(shift.arbeitsschichten_id);
    const departmentName = this.getDepartmentName(shiftInfo?.department_id);
    const stateColor = this.getStateColor(shift.state);

    const dialogRef = this.dialog.open(AssignEmployeesDialogComponent, {
      width: '600px',
      maxWidth: '90vw',
      height: 'auto',
      maxHeight: '90vh',
      disableClose: false,
      hasBackdrop: true,
      backdropClass: 'assign-employees-backdrop',
      panelClass: 'assign-employees-panel',
      autoFocus: 'first-tabbable',
      restoreFocus: true,
      enterAnimationDuration: 200,
      exitAnimationDuration: 200,
      data: {
        assignedShift: shift,
        shiftInfo: shiftInfo,
        departmentName: departmentName,
        stateColor: stateColor
      }
    });

    dialogRef.afterClosed().subscribe(result => {
      if (result) {
        this.loadData();
      }
    });
  }

  onEditAssignment(element: AssignedShift): void {
    // TODO: Implement edit assignment functionality
    console.log('Edit assignment:', element);
  }

  onDeleteAssignment(id: string): void {
    // TODO: Implement delete assignment functionality
    console.log('Delete assignment:', id);
  }
}
