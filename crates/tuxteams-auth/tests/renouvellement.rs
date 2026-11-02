//! Refresh grant contre un faux serveur HTTP (wiremock) : aucun accès réseau réel.

mod commun;

use tuxteams_application::ports::jetons::{ErreurAuth, PortRenouvellement, SecretRefresh};
use tuxteams_auth::{ClientDeviceCode, ConfigAuth};
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SUCCES: &str = include_str!("fixtures/renouvellement/succes.json");
const INVALID_GRANT: &str = include_str!("fixtures/renouvellement/invalid_grant.json");
const CHEMIN_JETON: &str = "/organizations/oauth2/v2.0/token";
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
        .and(path(CHEMIN_JETON))
        .respond_with(
            ResponseTemplate::new(statut).set_body_raw(corps.to_owned(), "application/json"),
        )
        .mount(serveur)
        .await;
}

async fn champs_recus(serveur: &MockServer) -> Vec<(String, String)> {
    let requetes = serveur.received_requests().await.expect("requêtes");
    let derniere = requetes.last().expect("une requête");
    url::form_urlencoded::parse(&derniere.body)
        .map(|(cle, valeur)| (cle.into_owned(), valeur.into_owned()))
        .collect()
}

fn champ(champs: &[(String, String)], nom: &str) -> Option<String> {
    champs
        .iter()
        .find(|(cle, _)| cle == nom)
        .map(|(_, valeur)| valeur.clone())
}

#[tokio::test]
async fn le_refresh_grant_envoie_le_refresh_token_le_client_id_et_les_scopes() {
    let serveur = MockServer::start().await;
    repondre(&serveur, 200, &commun::reponse_jeton(SUCCES)).await;

    client(&serveur).refresh(&secret()).await.expect("succès");

    let champs = champs_recus(&serveur).await;
    assert_eq!(
        champ(&champs, "grant_type").as_deref(),
        Some("refresh_token")
    );
    assert_eq!(
        champ(&champs, "refresh_token").as_deref(),
        Some(VALEUR_ENVOYEE)
    );
    assert_eq!(
        champ(&champs, "client_id").as_deref(),
        Some("1fec8e78-bce4-4aaf-ab1b-5451cc387264")
    );
    assert!(
        champ(&champs, "scope")
            .unwrap_or_default()
            .contains("offline_access"),
        "{champs:?}"
    );
}

#[tokio::test]
async fn un_refresh_reussi_rend_le_compte_et_le_refresh_token_en_rotation() {
    let serveur = MockServer::start().await;
    repondre(&serveur, 200, &commun::reponse_jeton(SUCCES)).await;

    let authentifie = client(&serveur).refresh(&secret()).await.expect("succès");

    assert_eq!(authentifie.compte.nom.as_deref(), Some("Alex Example"));
    assert_eq!(
        authentifie.compte.tenant,
        "00000000-0000-0000-0000-0000000000aa"
    );
    let rotation = authentifie.refresh.expect("refresh token en rotation");
    assert_eq!(rotation.exposer(), commun::VALEUR_RENOUVELLEMENT_FICTIVE);
}

#[tokio::test]
async fn invalid_grant_est_signale_comme_session_revoquee() {
    let serveur = MockServer::start().await;
    repondre(&serveur, 400, INVALID_GRANT).await;

    let resultat = client(&serveur).refresh(&secret()).await;

    assert_eq!(resultat, Err(ErreurAuth::SessionRevoquee));
}

#[tokio::test]
async fn une_reponse_sans_id_token_est_une_erreur_typee() {
    let serveur = MockServer::start().await;
    repondre(&serveur, 200, &commun::reponse_sans_id_token()).await;

    let resultat = client(&serveur).refresh(&secret()).await;

    assert!(
        matches!(resultat, Err(ErreurAuth::Inattendu(_))),
        "{resultat:?}"
    );
}

#[tokio::test]
async fn une_reponse_illisible_est_une_erreur_typee_sans_panique() {
    let serveur = MockServer::start().await;
    repondre(&serveur, 502, "<html>Bad gateway</html>").await;

    let resultat = client(&serveur).refresh(&secret()).await;

    assert!(
        matches!(resultat, Err(ErreurAuth::Inattendu(_))),
        "{resultat:?}"
    );
}

#[tokio::test]
async fn une_reponse_au_dela_de_la_limite_est_une_erreur_typee() {
    let serveur = MockServer::start().await;
    let corps = commun::reponse_jeton_volumineuse(SUCCES, 100 * 1024);
    repondre(&serveur, 200, &corps).await;

    let resultat = client(&serveur).refresh(&secret()).await;

    assert!(
        matches!(resultat, Err(ErreurAuth::Inattendu(_))),
        "{resultat:?}"
    );
}

#[tokio::test]
async fn un_serveur_injoignable_donne_une_erreur_reseau() {
    // Port réservé puis libéré : plus personne n'y écoute.
    let ecouteur = std::net::TcpListener::bind("127.0.0.1:0").expect("port libre");
    let adresse = ecouteur.local_addr().expect("adresse");
    drop(ecouteur);
    let config = ConfigAuth::par_defaut()
        .avec_autorite(Url::parse(&format!("http://{adresse}")).expect("url"));
    let client = ClientDeviceCode::new(config).expect("client créable");

    let resultat = client.refresh(&secret()).await;

    assert_eq!(resultat, Err(ErreurAuth::Reseau));
}
