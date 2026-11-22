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

    fn is_mother(&self) -> &'static str {
        "Esta sesión es la madre de la categoría: coordina la categoría y es la única que escribe el tablero."
    }

    fn mother_is(&self, entry: &str) -> String {
        format!("Madre de la categoría: {entry}.")
    }

    fn no_mother(&self) -> &'static str {
        "La categoría no tiene madre: sus sesiones son hermanas y coordinan directamente entre sí."
    }

    fn children_heading(&self) -> &'static str {
        "Hijas:"
    }

    fn no_children(&self) -> &'static str {
        "La categoría todavía no tiene sesiones hijas."
    }

    fn sisters_heading(&self) -> &'static str {
        "Hermanas:"
    }

    fn no_sisters(&self) -> &'static str {
        "Esta sesión todavía no tiene hermanas."
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

    fn child_reporting(&self) -> &'static str {
        "Al terminar una tarea o al bloquearse, esta sesión reporta a la madre; lo que esté fuera de su rol pasa por la madre."
    }

    fn board_of_mother(&self, board: &str) -> String {
        format!("Tablero de la categoría (solo lo escribe esta sesión): {board}")
    }

    fn board_of_child(&self, board: &str) -> String {
        format!(
            "Tablero de la categoría (lo mantiene la madre; de solo lectura para esta sesión salvo que la madre indique otra cosa): {board}"
        )
    }

    fn board_shared(&self, board: &str) -> String {
        format!("Tablero de la categoría (compartido entre las hermanas): {board}")
    }

    fn snapshot_note(&self, category_slug: &str) -> String {
        format!(
            "Esta lista es del momento del arranque; `lithemux status {category_slug}` muestra el estado actual."
        )
    }
}
