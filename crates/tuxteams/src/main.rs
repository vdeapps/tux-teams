//! Binaire TuxTeams : composition et ouverture de la fenêtre.

use slint::ComponentHandle;
use tuxteams_ui::Fenetre;

fn main() -> Result<(), slint::PlatformError> {
    Fenetre::new()?.run()
}
