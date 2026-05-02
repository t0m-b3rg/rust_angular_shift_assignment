use crate::models::AssignedShiftState;

// Helper: Convert HH:MM to minutes since midnight
pub fn time_to_minutes(time_str: &str) -> i32 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(h), Ok(m)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
            return h * 60 + m;
        }
    }
    0
}

// Helper: Check if two time ranges overlap (handles overnight shifts)
pub fn times_overlap(start1: &str, end1: &str, start2: &str, end2: &str) -> bool {
    let s1 = time_to_minutes(start1);
    let e1 = time_to_minutes(end1);
    let s2 = time_to_minutes(start2);
    let e2 = time_to_minutes(end2);

    // Detect overnight shifts (end < start, e.g., 22:00 to 06:00)
    let is_overnight1 = e1 < s1;
    let is_overnight2 = e2 < s2;

    match (is_overnight1, is_overnight2) {
        (true, true) => {
            // Both are overnight shifts on the same day - they ALWAYS overlap
            // Both span evening-to-midnight and midnight-to-morning portions
            true
        }
        (true, false) => {
            // Shift 1 is overnight (22:00-06:00), Shift 2 is normal
            // Overlap if shift 2 starts before shift 1 ends OR shift 2 ends after shift 1 starts
            s2 < e1 || s1 < e2
        }
        (false, true) => {
            // Shift 2 is overnight, Shift 1 is normal
            s1 < e2 || s2 < e1
        }
        (false, false) => {
            // Both are normal shifts
            s1 < e2 && s2 < e1
        }
    }
}

// Helper macro for debug logging
#[macro_export]
macro_rules! debug_log {
    ($debug:expr, $($arg:tt)*) => {
        if $debug {
            println!($($arg)*);
        }
    };
}

// Helper: Determine AssignedShift state based on employee count and work hours availability
pub fn determine_assigned_shift_state(
    employee_count: usize,
    required_hours: i32,
    override_state: Option<AssignedShiftState>,
    debug: bool,
) -> AssignedShiftState {
    if employee_count == 0 {
        debug_log!(debug, "[SHIFT STATE] No employees assigned → Setting state to Unassigned");
        return AssignedShiftState::Unassigned;
    }

    let available_hours = employee_count as i32 * 8;
    
    if available_hours < required_hours {
        debug_log!(debug, "[SHIFT STATE] Insufficient work hours: {} employees × 8 = {} hours, but {} hours required → Setting state to WorkHoursNotSatisfied", 
            employee_count, available_hours, required_hours);
        return AssignedShiftState::WorkHoursNotSatisfied;
    }

    // Sufficient employees and hours - use override state if provided, otherwise Valid
    if let Some(override_state_val) = override_state {
        debug_log!(debug, "[SHIFT STATE] Sufficient hours ({} employees, {} available vs {} required) → Using provided state: {}", 
            employee_count, available_hours, required_hours, override_state_val.as_str());
        override_state_val
    } else {
        debug_log!(debug, "[SHIFT STATE] Sufficient hours ({} employees, {} available vs {} required) → Setting state to Valid", 
            employee_count, available_hours, required_hours);
        AssignedShiftState::Valid
    }
}
