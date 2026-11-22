use crate::modules::context::domain::templates::phrases::Phrases;
use crate::modules::session::{ActivityAge, State};

pub struct Es;

impl Es {
    fn state(state: State) -> &'static str {
        match state {
            State::New => "nueva",
            State::Working => "trabajando",
            State::Idle => "inactiva",
            State::Waiting => "esperando",
            State::Hibernated => "hibernada",
            State::Ended => "terminada",
        }
    }

    fn last_activity(age: Option<ActivityAge>) -> String {
        let ago = match age {
            None => return "sin actividad todavía".to_string(),
            Some(ActivityAge::JustNow) => "hace un momento".to_string(),
            Some(ActivityAge::Minutes(minutes)) => format!("hace {minutes} min"),
            Some(ActivityAge::Hours(hours)) => format!("hace {hours} h"),
            Some(ActivityAge::Days(1)) => "hace 1 día".to_string(),
            Some(ActivityAge::Days(days)) => format!("hace {days} días"),
        };
        format!("última actividad {ago}")
    }
}

impl Phrases for Es {
    fn belongs_to(&self, category_name: &str, root_dir: &str) -> String {
        format!(
            "lithemux: esta sesión pertenece a la categoría «{category_name}» (carpeta raíz {root_dir})."
        )
    }

    fn identity(&self, name: &str, role: &str) -> String {
        format!("Nombre de esta sesión: {name}. Rol: {role}.")
    }

    fn is_coordinator(&self) -> &'static str {
        "Esta sesión es la sesión coordinadora de la categoría: coordina la categoría y es la única que escribe el tablero."
    }

    fn coordinator_is(&self, entry: &str) -> String {
        format!("Coordinadora de la categoría: {entry}.")
    }

    fn no_coordinator(&self) -> &'static str {
        "La categoría no tiene sesión coordinadora: sus sesiones coordinan directamente entre sí."
    }

    fn members_heading(&self) -> &'static str {
        "Las demás sesiones:"
    }

    fn no_members(&self) -> &'static str {
        "La categoría todavía no tiene más sesiones."
    }

    fn others_heading(&self) -> &'static str {
        "Las demás sesiones:"
    }

    fn no_others(&self) -> &'static str {
        "La categoría todavía no tiene más sesiones."
    }

    fn more(&self, hidden: usize) -> String {
        format!("… y {hidden} más")
    }

    fn entry(
        &self,
        name: &str,
        role: &str,
        state: State,
        age: Option<ActivityAge>,
        shell: bool,
    ) -> String {
        let shell = if shell {
            "terminal de shell, sin SendMessage; "
        } else {
            ""
        };
        format!(
            "{name} ({shell}rol: {role}; estado: {}, {})",
            Self::state(state),
            Self::last_activity(age)
        )
    }

    fn messaging(&self) -> &'static str {
        "Las sesiones se comunican con SendMessage usando esos nombres."
    }

    fn member_reporting(&self) -> &'static str {
        "Al terminar una tarea o al bloquearse, esta sesión reporta a la coordinadora; lo que esté fuera de su rol pasa por la coordinadora."
    }

    fn board_of_coordinator(&self, board: &str) -> String {
        format!("Tablero de la categoría (solo lo escribe esta sesión): {board}")
    }

    fn board_of_member(&self, board: &str) -> String {
        format!(
            "Tablero de la categoría (lo mantiene la coordinadora; de solo lectura para esta sesión salvo que la coordinadora indique otra cosa): {board}"
        )
    }

    fn board_shared(&self, board: &str) -> String {
        format!("Tablero de la categoría (compartido entre sus sesiones): {board}")
    }

    fn snapshot_note(&self, category_slug: &str) -> String {
        format!(
            "Esta lista es del momento del arranque; `lithemux status {category_slug}` muestra el estado actual."
        )
    }
}
