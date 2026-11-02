//! Aides de test : assemble les réponses du point de terminaison de jeton à l'exécution,
//! pour ne stocker aucun jeton (ni JWT encodé) dans les fixtures.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};

const VALEUR_ACCES_FICTIVE: &str = "valeur-acces-fictive";
const VALEUR_RENOUVELLEMENT_FICTIVE: &str = "valeur-renouvellement-fictive";

/// JWT non signé `en-tête.charge.` construit depuis des claims en clair.
fn jwt_non_signe(claims: &Value) -> String {
    let entete = URL_SAFE_NO_PAD.encode(json!({"alg": "none", "typ": "JWT"}).to_string());
    let charge = URL_SAFE_NO_PAD.encode(claims.to_string());
    format!("{entete}.{charge}.")
}

/// Réponse de jeton complète : la fixture porte les claims en clair (`id_token_claims`),
/// les valeurs d'accès et de renouvellement viennent de constantes de test.
pub fn reponse_jeton(fixture: &str) -> String {
    let mut reponse: Value = serde_json::from_str(fixture).expect("fixture JSON valide");
    let objet = reponse.as_object_mut().expect("fixture : objet JSON");
    let claims = objet
        .remove("id_token_claims")
        .expect("fixture : id_token_claims");
    objet.insert("access_token".into(), json!(VALEUR_ACCES_FICTIVE));
    objet.insert("refresh_token".into(), json!(VALEUR_RENOUVELLEMENT_FICTIVE));
    objet.insert("id_token".into(), json!(jwt_non_signe(&claims)));
    reponse.to_string()
}

/// Réponse de jeton réduite à la valeur d'accès (sans id_token).
pub fn reponse_sans_id_token() -> String {
    json!({"access_token": VALEUR_ACCES_FICTIVE}).to_string()
}

/// Réponse de jeton complète dont le corps est gonflé d'un champ de remplissage d'environ
/// `taille` octets (cas d'un jeton d'accès volumineux).
pub fn reponse_jeton_volumineuse(fixture: &str, taille: usize) -> String {
    let mut reponse: Value = serde_json::from_str(&reponse_jeton(fixture)).expect("JSON valide");
    reponse
        .as_object_mut()
        .expect("objet JSON")
        .insert("remplissage".into(), json!("x".repeat(taille)));
    reponse.to_string()
}
