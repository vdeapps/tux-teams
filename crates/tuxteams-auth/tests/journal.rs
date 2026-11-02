//! Journalisation du refresh grant. Fichier à part : `tracing` met en cache l'intérêt de chaque
//! point de journalisation, ce qui rend la capture instable si d'autres tests du même processus
//! exécutent les mêmes points en parallèle.

mod commun;

use std::io;
use std::sync::{Arc, Mutex};

use tuxteams_application::ports::jetons::{PortRenouvellement, SecretRefresh};
use tuxteams_auth::{ClientDeviceCode, ConfigAuth};
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SUCCES: &str = include_str!("fixtures/renouvellement/succes.json");
const INVALID_GRANT: &str = include_str!("fixtures/renouvellement/invalid_grant.json");
const VALEUR_ENVOYEE: &str = "valeur-envoyee-fictive";

fn client(serveur: &MockServer) -> ClientDeviceCode {
    let config =
        ConfigAuth::par_defaut().avec_autorite(Url::parse(&serveur.uri()).expect("url du serveur"));
    ClientDeviceCode::new(config).expect("client créable")
}

fn secret() -> SecretRefresh {
    SecretRefresh::new(VALEUR_ENVOYEE.to_owned())
}

async fn repondre(serveur: &MockServer, statut: u16, corps: &str) {
    Mock::given(method("POST"))
        .and(path("/organizations/oauth2/v2.0/token"))
        .respond_with(
            ResponseTemplate::new(statut).set_body_raw(corps.to_owned(), "application/json"),
        )
        .mount(serveur)
        .await;
}

/// Écrivain partagé qui capture tout ce que `tracing` émet.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl io::Write for Capture {
    fn write(&mut self, octets: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("verrou").extend_from_slice(octets);
        Ok(octets.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn le_refresh_grant_ne_journalise_aucun_jeton() {
    let capture = Capture::default();
    let ecrivain = capture.clone();
    let abonne = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(move || ecrivain.clone())
        .finish();
    let _garde = tracing::subscriber::set_default(abonne);

    // Réponses réussie, révoquée, hors limite et inattendue : toutes les branches qui journalisent.
    let ok = MockServer::start().await;
    repondre(&ok, 200, &commun::reponse_jeton(SUCCES)).await;
    let _ = client(&ok).refresh(&secret()).await;
    let revoque = MockServer::start().await;
    repondre(&revoque, 400, INVALID_GRANT).await;
    let _ = client(&revoque).refresh(&secret()).await;
    let gros = MockServer::start().await;
    repondre(
        &gros,
        200,
        &commun::reponse_jeton_volumineuse(SUCCES, 100 * 1024),
    )
    .await;
    let _ = client(&gros).refresh(&secret()).await;

    let journal = String::from_utf8(capture.0.lock().expect("verrou").clone()).expect("utf8");
    assert!(
        journal.contains("réponse inattendue"),
        "le journal doit contenir l'avertissement : {journal:?}"
    );
    for valeur in [
        VALEUR_ENVOYEE,
        commun::VALEUR_RENOUVELLEMENT_FICTIVE,
        commun::VALEUR_ACCES_FICTIVE,
        "Alex",
        "contoso",
    ] {
        assert!(
            !journal.contains(valeur),
            "donnée sensible journalisée : {journal}"
        );
    }
}
