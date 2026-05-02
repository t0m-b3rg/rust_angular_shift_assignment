import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

const routes: Routes = [
  {
    path: '',
    redirectTo: 'dashboard',
    pathMatch: 'full'
  },
  {
    path: 'dashboard',
    loadChildren: () => import('./modules/dashboard/dashboard.module').then(m => m.DashboardModule)
  },
  {
    path: 'employees',
    loadChildren: () => import('./modules/employees/employees.module').then(m => m.EmployeesModule)
  },
  {
    path: 'departments',
    loadChildren: () => import('./modules/departments/departments.module').then(m => m.DepartmentsModule)
  },
  {
    path: 'shifts',
    loadChildren: () => import('./modules/shifts/shifts.module').then(m => m.ShiftsModule)
  },
  {
    path: 'schedule',
    loadChildren: () => import('./modules/schedule/schedule.module').then(m => m.ScheduleModule)
  },
  {
    path: '**',
    redirectTo: 'dashboard'
  }
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule { }
