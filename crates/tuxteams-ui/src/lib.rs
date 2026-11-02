//! Interface Slint de TuxTeams : fenêtre, rail, tokens de design.
//!
//! Composants minces : aucune logique métier ici, uniquement de la présentation.

pub mod theme;

/// Code généré par Slint : les lints `clippy::unwrap_used` s'y appliquent à du code que nous
/// n'écrivons pas ; l'exception est limitée à ce module.
#[allow(clippy::unwrap_used)]
mod genere {
    slint::include_modules!();
}

pub use genere::*;
