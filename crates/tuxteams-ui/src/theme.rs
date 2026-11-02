//! Calculs liés aux tokens de design (contraste WCAG).
//!
//! La source unique des couleurs est `ui/theme.slint` (global `Theme`).

use slint::Color;

/// Linéarise un canal sRGB (0-255), formule WCAG 2.x.
fn canal_lineaire(canal: u8) -> f64 {
    let valeur = f64::from(canal) / 255.0;
    if valeur <= 0.04045 {
        valeur / 12.92
    } else {
        ((valeur + 0.055) / 1.055).powf(2.4)
    }
}

/// Luminance relative d'une couleur (WCAG 2.x), entre 0 (noir) et 1 (blanc).
fn luminance_relative(couleur: Color) -> f64 {
    0.2126 * canal_lineaire(couleur.red())
        + 0.7152 * canal_lineaire(couleur.green())
        + 0.0722 * canal_lineaire(couleur.blue())
}

/// Ratio de contraste WCAG 2.x entre deux couleurs, de 1 (identiques) à 21 (noir sur blanc).
pub fn ratio_contraste(premiere: Color, seconde: Color) -> f64 {
    let (a, b) = (luminance_relative(premiere), luminance_relative(seconde));
    let (clair, sombre) = if a >= b { (a, b) } else { (b, a) };
    (clair + 0.05) / (sombre + 0.05)
}
