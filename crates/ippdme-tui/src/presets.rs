//! Postman-style preset command collections, grouped by category.

use ippdme_core::{Command, CoordSystem};

#[derive(Clone)]
pub struct Preset {
    pub label: &'static str,
    pub build: fn() -> Command,
}

#[derive(Clone)]
pub struct PresetCategory {
    pub name: &'static str,
    pub items: Vec<Preset>,
}

pub fn default_presets() -> Vec<PresetCategory> {
    vec![
        PresetCategory {
            name: "Session",
            items: vec![
                Preset {
                    label: "StartSession()",
                    build: || Command::StartSession,
                },
                Preset {
                    label: "EndSession()",
                    build: || Command::EndSession,
                },
                Preset {
                    label: "GetDMEVersion()",
                    build: || Command::GetDmeVersion,
                },
            ],
        },
        PresetCategory {
            name: "Motion",
            items: vec![
                Preset {
                    label: "Home()",
                    build: || Command::Home,
                },
                Preset {
                    label: "GoTo(10, 10, 10)",
                    build: || Command::go_to(10.0, 10.0, 10.0),
                },
                Preset {
                    label: "GoTo(0, 0, 0)",
                    build: || Command::go_to(0.0, 0.0, 0.0),
                },
            ],
        },
        PresetCategory {
            name: "Measurement",
            items: vec![Preset {
                label: "PtMeas()",
                build: Command::pt_meas,
            }],
        },
        PresetCategory {
            name: "Tooling",
            items: vec![
                Preset {
                    label: "SetCoordSystem(MCS)",
                    build: || Command::SetCoordSystem(CoordSystem::Mcs),
                },
                Preset {
                    label: "SetCoordSystem(PCS)",
                    build: || Command::SetCoordSystem(CoordSystem::Pcs),
                },
            ],
        },
        PresetCategory {
            name: "Status",
            items: vec![
                Preset {
                    label: "IsHomed()",
                    build: || Command::IsHomed,
                },
                Preset {
                    label: "IsUserEnabled()",
                    build: || Command::IsUserEnabled,
                },
                Preset {
                    label: "EnableUser()",
                    build: || Command::EnableUser,
                },
                Preset {
                    label: "DisableUser()",
                    build: || Command::DisableUser,
                },
                Preset {
                    label: "GetMachineClass()",
                    build: || Command::GetMachineClass,
                },
                Preset {
                    label: "GetErrStatusE()",
                    build: || Command::GetErrStatusE,
                },
                Preset {
                    label: "GetXtdErrStatus()",
                    build: || Command::GetXtdErrStatus,
                },
                Preset {
                    label: "Get(X, Y, Z)",
                    build: || {
                        Command::Get(vec![
                            ippdme_core::Term::unit("X"),
                            ippdme_core::Term::unit("Y"),
                            ippdme_core::Term::unit("Z"),
                        ])
                    },
                },
            ],
        },
        PresetCategory {
            name: "Errors & Daemons",
            items: vec![
                Preset {
                    label: "ClearAllErrors()",
                    build: || Command::ClearAllErrors,
                },
                Preset {
                    label: "AbortE()",
                    build: || Command::AbortE,
                },
                Preset {
                    label: "StopAllDaemons()",
                    build: || Command::StopAllDaemons,
                },
            ],
        },
    ]
}

/// Flatten categories into `(category_index, item_index, category_name, preset)`
/// tuples for simple linear list navigation.
pub fn flatten(categories: &[PresetCategory]) -> Vec<(usize, usize, &'static str, Preset)> {
    let mut out = Vec::new();
    for (ci, cat) in categories.iter().enumerate() {
        for (ii, item) in cat.items.iter().enumerate() {
            out.push((ci, ii, cat.name, item.clone()));
        }
    }
    out
}
