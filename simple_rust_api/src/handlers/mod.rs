pub mod common;
pub mod departments;
pub mod employees;
pub mod shifts;
pub mod assigned_shifts;

// Re-export handler functions for easy access
pub use departments::{create_department, list_departments, get_department, update_department, delete_department};
pub use employees::{create_employee, list_employees, get_employee, update_employee, delete_employee};
pub use shifts::{create_shift, list_shifts, get_shift, update_shift, delete_shift, get_department_shifts};
pub use assigned_shifts::{
    create_assigned_shift, list_assigned_shifts, get_assigned_shift, update_assigned_shift,
    delete_assigned_shift, get_department_assigned_shifts, init_assigned_shifts,
};
