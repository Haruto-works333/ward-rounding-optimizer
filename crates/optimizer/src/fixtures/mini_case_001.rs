use domain::{
    Minute, Point, Priority, ProblemInput, Room, ScoringWeights, Staff, StaffRole, Task,
    TaskRequirement, TimeWindow,
};

pub fn mini_case_001(nurse_count: usize) -> ProblemInput {
    let rooms = vec![
        Room::new("R101", "R101", 1, 0),
        Room::new("R102", "R102", 2, 0),
        Room::new("R103", "R103", 3, 0),
        Room::new("R104", "R104", 1, 2),
        Room::new("R105", "R105", 2, 2),
        Room::new("R106", "R106", 3, 2),
    ];

    let tasks = vec![
        Task::new(
            "T1",
            "R101",
            TaskRequirement::DoctorRequired,
            6,
            500,
            Priority::High,
        ),
        Task::new(
            "T2",
            "R102",
            TaskRequirement::NurseCapable,
            5,
            300,
            Priority::Normal,
        ),
        Task::new(
            "T3",
            "R103",
            TaskRequirement::NurseOnly,
            4,
            250,
            Priority::Normal,
        ),
        Task::new(
            "T4",
            "R104",
            TaskRequirement::DoctorAndNurseSync,
            8,
            700,
            Priority::High,
        ),
        Task::new(
            "T5",
            "R105",
            TaskRequirement::NurseCapable,
            5,
            300,
            Priority::Normal,
        ),
        Task::new(
            "T6",
            "R106",
            TaskRequirement::DoctorRequired,
            6,
            450,
            Priority::Normal,
        ),
        Task::new(
            "T7",
            "R101",
            TaskRequirement::NurseOnly,
            3,
            150,
            Priority::Low,
        ),
        Task::new(
            "T8",
            "R103",
            TaskRequirement::NurseCapable,
            4,
            220,
            Priority::Low,
        ),
        Task::new(
            "T9",
            "R105",
            TaskRequirement::DoctorAndNurseSync,
            7,
            650,
            Priority::High,
        ),
        Task::new(
            "T10",
            "R106",
            TaskRequirement::NurseOnly,
            4,
            180,
            Priority::Low,
        ),
    ];

    let mut staff = vec![Staff::new(
        "D1",
        "Doctor 1",
        StaffRole::Doctor,
        Minute::new(0),
        Minute::new(30),
    )];

    for index in 1..=nurse_count {
        staff.push(Staff::new(
            format!("N{index}"),
            format!("Nurse {index}"),
            StaffRole::Nurse,
            Minute::new(0),
            Minute::new(30),
        ));
    }

    ProblemInput {
        rooms,
        tasks,
        staff,
        depot: Point::new(0, 0),
        planning_window: TimeWindow::new(Minute::new(0), Minute::new(30)),
        scoring_weights: ScoringWeights::default(),
    }
}
