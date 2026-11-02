//! Coffre contre un vrai Secret Service de session D-Bus. Exécuté par `make test-integration`
//! (feature `integration`), jamais par `make test`.
//!
//! Isolation : collection « session » (en mémoire, distincte du trousseau de connexion de
//! l'utilisateur) et attribut `application = tuxteams-test` : aucun secret réel n'est lu ni supprimé.
#![cfg(feature = "integration")]

use oo7::{Keyring, dbus};
use tuxteams_application::ports::coffre::PortCoffre;
use tuxteams_application::ports::jetons::SecretRefresh;
use tuxteams_auth::CoffreSecretService;
use tuxteams_domaine::session::Compte;

const APPLICATION_DE_TEST: &str = "tuxteams-test";
const PREMIERE_VALEUR: &str = "valeur-integration-un";
const SECONDE_VALEUR: &str = "valeur-integration-deux";

async fn coffre_de_test() -> CoffreSecretService {
    let service = dbus::Service::new()
        .await
        .expect("Secret Service de session joignable");
    let collection = service
        .session_collection()
        .await
        .expect("collection de session");
    CoffreSecretService::avec_trousseau(Keyring::DBus(collection), APPLICATION_DE_TEST)
}

#[tokio::test]
async fn un_secret_enregistre_est_relu_puis_supprime() {
    let coffre = coffre_de_test().await;
    let compte =
        Compte::depuis_identite("tenant-test", Some("Alex Example"), None).expect("compte");
    coffre.delete_refresh().await.expect("nettoyage initial");

    let avant = coffre.load_refresh().await;
    let enregistre = coffre
        .save_refresh(&compte, &SecretRefresh::new(PREMIERE_VALEUR.to_owned()))
        .await;
    let relu = coffre.load_refresh().await;
    let remplace = coffre
        .save_refresh(&compte, &SecretRefresh::new(SECONDE_VALEUR.to_owned()))
        .await;
    let relu_apres_rotation = coffre.load_refresh().await;
    let suppression = coffre.delete_refresh().await;
    let apres = coffre.load_refresh().await;
    // Nettoyage garanti avant les assertions, même si l'une d'elles échoue.
    coffre.delete_refresh().await.expect("nettoyage final");

    assert_eq!(avant.expect("lecture initiale"), None);
    enregistre.expect("enregistrement");
    assert_eq!(
        relu.expect("relecture").map(|s| s.exposer().to_owned()),
        Some(PREMIERE_VALEUR.to_owned())
    );
    remplace.expect("remplacement");
    assert_eq!(
        relu_apres_rotation
            .expect("relecture après rotation")
            .map(|s| s.exposer().to_owned()),
        Some(SECONDE_VALEUR.to_owned()),
        "la rotation remplace la valeur sans doublon"
    );
    suppression.expect("suppression");
    assert_eq!(apres.expect("lecture finale"), None);
}
