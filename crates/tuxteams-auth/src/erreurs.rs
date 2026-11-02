//! Traduction des erreurs OAuth du service d'identité vers les erreurs du port.

use serde::Deserialize;
use tuxteams_application::ports::jetons::ErreurAuth;

#[derive(Deserialize)]
struct CorpsErreur {
    error: Option<String>,
    error_description: Option<String>,
}

/// Interprète un corps d'erreur OAuth. Seul le code d'erreur (assaini) est conservé : la
/// description peut contenir des identifiants de corrélation ou de compte.
pub(crate) fn depuis_corps(corps: &[u8]) -> ErreurAuth {
    let Ok(erreur) = serde_json::from_slice::<CorpsErreur>(corps) else {
        return ErreurAuth::Inattendu("corps_illisible".to_owned());
    };
    if erreur
        .error_description
        .as_deref()
        .is_some_and(|description| description.contains("AADSTS53003"))
    {
        return ErreurAuth::AccesConditionnel;
    }
    match erreur.error.as_deref() {
        Some("expired_token" | "code_expired") => ErreurAuth::CodeExpire,
        Some("authorization_declined" | "access_denied") => ErreurAuth::Refuse,
        Some(code) if code_sur(code) => ErreurAuth::Inattendu(code.to_owned()),
        _ => ErreurAuth::Inattendu("inconnu".to_owned()),
    }
}

fn code_sur(code: &str) -> bool {
    !code.is_empty() && code.len() <= 64 && code.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

/// Code d'erreur OAuth brut d'un corps de réponse, s'il y en a un.
pub(crate) fn code_oauth(corps: &[u8]) -> Option<String> {
    serde_json::from_slice::<CorpsErreur>(corps).ok()?.error
}
