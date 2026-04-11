use crate::hydra::HydraBuild;

#[derive(Debug, Clone, Copy)]
pub struct StatusInfo {
    pub icon_variant: &'static str,
    pub color: &'static str,
    pub label: &'static str,
}

pub fn status_info(build: &HydraBuild) -> StatusInfo {
    if build.finished == 0 {
        return StatusInfo {
            icon_variant: "question",
            color: "#A6AEB0",
            label: "BUILDING...",
        };
    }

    match build.buildstatus.unwrap_or(1) {
        0 => StatusInfo {
            icon_variant: "checkmark",
            color: "#61B329",
            label: "SUCCESS",
        },
        1 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "FAILED",
        },
        2 => StatusInfo {
            icon_variant: "gray-x",
            color: "#4D5357",
            label: "DEPENDENCY FAILED",
        },
        3 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "ABORTED",
        },
        4 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "CANCELLED",
        },
        6 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "FAILED WITH OUTPUT",
        },
        7 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "TIMED OUT",
        },
        9 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "UNSUPPORTED SYSTEM",
        },
        10 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "LOG LIMIT EXCEEDED",
        },
        11 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "OUTPUT LIMIT EXCEEDED",
        },
        12 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "NON-DETERMINISTIC",
        },
        _ => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "FAILED",
        },
    }
}

// Got these from Hydra's repo :)
pub fn icon_svg(icon_variant: &str) -> &'static str {
    match icon_variant {
        "checkmark" => {
            r##"<svg width="172" height="172" viewBox="0 0 64 64" aria-hidden="true"><path fill="#61B329" d="M55.999 2L18.8 42.909 8 34.729H2L18.8 62 62 2z"/></svg>"##
        }
        "gray-x" => {
            r##"<svg width="172" height="172" viewBox="0 0 64 64" aria-hidden="true"><path fill="#4D5357" d="M62 10.571L53.429 2 32 23.429 10.571 2 2 10.571 23.429 32 2 53.429 10.571 62 32 40.571 53.429 62 62 53.429 40.571 32z"/></svg>"##
        }
        "stopsign" => {
            r##"<svg width="172" height="172" viewBox="0 0 64 64" aria-hidden="true"><path fill="#E9EDF2" d="M64 45.254L45.254 64H18.747L0 45.254V18.747L18.747 0h26.507L64 18.747z"/><path fill="#ED4C5C" d="M58 42.768L42.769 58H21.231L6 42.768V21.231L21.231 6h21.538L58 21.231z"/></svg>"##
        }
        "question" => {
            r##"<svg width="172" height="172" viewBox="0 0 64 64" aria-hidden="true"><g fill-rule="evenodd" clip-rule="evenodd" fill="#A6AEB0"><path d="M30.249 2.065C18.612 2.789 12.531 9.379 12 21.296h11.739c.147-4.128 2.451-7.214 6.741-7.669 4.211-.447 8.206.556 9.416 3.435 1.307 3.11-1.627 6.724-3.022 8.241-2.582 2.813-6.776 4.865-8.95 7.9-2.131 2.974-2.51 6.887-2.674 11.676h10.346c.145-3.062.349-5.995 1.742-7.898 2.266-3.092 5.65-4.541 8.486-6.983 2.709-2.334 5.559-5.147 6.043-9.501C53.319 7.466 42.683 1.289 30.249 2.065z"/><ellipse cx="30.515" cy="55.567" rx="6.532" ry="6.433"/></g></svg>"##
        }
        _ => {
            r##"<svg width="172" height="172" viewBox="0 0 64 64" aria-hidden="true"><path fill="#FF5A79" d="M62 10.571L53.429 2 32 23.429 10.571 2 2 10.571 23.429 32 2 53.429 10.571 62 32 40.571 53.429 62 62 53.429 40.571 32z"/></svg>"##
        }
    }
}
